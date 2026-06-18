use std::{collections::HashSet, ops::Mul};

use common_game::components::resource::{
    BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource, ResourceType,
};
use crate::config::NEEDS_MAGIC_NUMBER;
use crate::galaxy::resources::resource_index;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResourceVector([u64; 10]);

impl Mul for ResourceVector {
    type Output = ResourceVector;

    /// Performs the element-wise multiplication between two `ResourceVector`.
    ///
    /// Returns the `ResourceVector` result
    fn mul(self, rhs: ResourceVector) -> Self::Output {
        ResourceVector(std::array::from_fn(|i| self.0[i] * rhs.0[i]))
    }
}

impl ResourceVector {
    /// Creates a new `ResourceVector` from an array.
    ///
    /// Returns the new `ResourceVector`
    pub(crate) fn new(vec: [u64; 10]) -> Self {
        ResourceVector(vec)
    }

    /// Creates a new `ResourceVector` initialized with all zeros.
    ///
    /// Returns the new `ResourceVector`
    pub(crate) fn zeros() -> Self {
        ResourceVector([0; 10])
    }

    /// Creates a new `ResourceVector` initialized with the specific needs for a `ComplexResourceType`
    ///
    /// Returns the new `ResourceVector`
    pub(crate) fn generate_complex_needs(complex: ComplexResourceType) -> Self {
        let n = NEEDS_MAGIC_NUMBER;
        match complex {
            ComplexResourceType::Diamond =>   ResourceVector([2,0,0,0,n,0,0,0,0,0]),
            ComplexResourceType::Water =>     ResourceVector([0,1,1,0,0,n,0,0,0,0]),
            ComplexResourceType::Life =>      ResourceVector([1,1,1,0,0,1,n,0,0,0]),
            ComplexResourceType::Robot =>     ResourceVector([1,1,1,1,0,1,1,n,0,0]),
            ComplexResourceType::Dolphin =>   ResourceVector([1,1,1,0,0,1,1,0,n,0]),
            ComplexResourceType::AIPartner => ResourceVector([2,1,1,1,0,1,1,1,0,n]),
        }
    }

    /// Generates a `ResourceVector` with a high value for the given `ResourceType`.
    pub(crate) fn generate_resource_needs(resource: ResourceType) -> Self {
        match resource {
            ResourceType::Basic(res) => {
                let mut vec = ResourceVector::zeros();
                vec.0[resource_index(ResourceType::Basic(res))] = NEEDS_MAGIC_NUMBER;
                vec
            }
            ResourceType::Complex(res) => {
                let mut vec = ResourceVector::generate_complex_needs(res);
                vec.0[resource_index(ResourceType::Complex(res))] = NEEDS_MAGIC_NUMBER;
                vec
            }
        }
    }

    /// Gets the internal array of a `ResourceVector`.
    ///
    /// Returns a `[u64; 10]`
    pub(crate) fn get(&self) -> [u64; 10] {
        self.0
    }

    /// Checks if a `ResourceVector` contains all zeros.
    ///
    /// Returns a `bool`
    pub(crate) fn is_zero(&self) -> bool {
        self.0 == [0; 10]
    }

    /// Performs the dot product between two `ResourceVector`.
    ///
    /// Returns the `u64` result.
    pub(crate) fn dot(&self, rhs: &ResourceVector) -> u64 {
        self.0.iter().zip(rhs.0.iter()).map(|(a, b)| a * b).sum()
    }

    /// Performs a softmax on the internal array and samples an element index from it.
    ///
    /// Returns the sampled `usize` index.
    pub fn softmax_sample(&self, temperature: f64) -> usize {
        #[allow(clippy::cast_precision_loss)]
        let scores: Vec<f64> = self.0.iter().map(|&x| x as f64).collect();
        let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = scores
            .iter()
            .map(|s| {
                if *s <= 0.0 {
                    0.0
                } else {
                    ((s - max) / temperature).exp()
                }
            })
            .collect();
        let sum: f64 = exps.iter().sum();
        let probs: Vec<f64> = exps.iter().map(|e| e / sum).collect();

        let mut r = rand::random::<f64>();
        probs
            .iter()
            .enumerate()
            .find(|(_, p)| {
                r -= *p;
                r <= 0.0
            })
            .map_or(0, |(i, _)| i)
    }

    /// Sets the basic resources of a `ResourceVector`, given the `HashSet<BasicResourceType>`.
    /// The basic resources correspond to the first four entries of the array.
    pub(crate) fn set_basic(&mut self, resources: &HashSet<BasicResourceType>) {
        self.clear_basic();
        for basic in resources {
            self.0[resource_index(ResourceType::Basic(*basic))] = 1;
        }
    }

    /// Sets the complex resources of a `ResourceVector`, given the `HashSet<ComplexResourceType>`.
    /// The complex resources correspond to the last six entries of the array.
    pub(crate) fn set_complex(&mut self, resources: &HashSet<ComplexResourceType>) {
        self.clear_complex();
        for complex in resources {
            self.0[resource_index(ResourceType::Complex(*complex))] = 1;
        }
    }

    /// Sets to zero the basic resources of a `ResourceVector`, which correspond to the first four entries
    pub(crate) fn clear_basic(&mut self) {
        self.0[0..4].fill(0);
    }

    /// Sets to zero the complex resources of a `ResourceVector`, which correspond to the last six entries
    pub(crate) fn clear_complex(&mut self) {
        self.0[4..10].fill(0);
    }

    /// Decreases by one the entry of the `ResourceVector` corresponding to the given `GenericResource`.
    pub(crate) fn decrease_need(&mut self, resource: &GenericResource) {
        match resource {
            GenericResource::BasicResources(basic) => {
                let idx = resource_index(ResourceType::Basic(basic.get_type()));
                self.0[idx] = self.0[idx].saturating_sub(1);
            }

            GenericResource::ComplexResources(complex) => {
                if complex.get_type() != ComplexResourceType::AIPartner {
                    let idx = resource_index(ResourceType::Complex(complex.get_type()));
                    if self.0[idx] != NEEDS_MAGIC_NUMBER {
                        self.0[idx] = self.0[idx].saturating_sub(1);
                    }
                }
            }
        }
    }

    /// Increases by one the entry of the `ResourceVector` corresponding to the given `GenericResource`.
    pub(crate) fn increase_needs(&mut self, complex_request: &ComplexResourceRequest) {
        let indexes: (usize, usize) = match complex_request {
            ComplexResourceRequest::Diamond(..) => (0, 0),
            ComplexResourceRequest::Water(..) => (1, 2),
            ComplexResourceRequest::Life(..) => (0, 5),
            ComplexResourceRequest::Robot(..) => (3, 6),
            ComplexResourceRequest::Dolphin(..) => (5, 6),
            ComplexResourceRequest::AIPartner(..) => (4, 7),
        };
        self.0[indexes.0] += 1;
        self.0[indexes.1] += 1;
    }
}
