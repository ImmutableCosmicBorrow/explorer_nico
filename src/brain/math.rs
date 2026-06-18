use std::{collections::HashSet, ops::Mul};

use common_game::components::resource::{BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource, ResourceType};

use crate::galaxy::resources::resource_index;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Vec10([u64; 10]);

impl Mul for Vec10 {
    type Output = Vec10;

    fn mul(self, rhs: Vec10) -> Self::Output {
        Vec10(std::array::from_fn(|i| self.0[i] * rhs.0[i]))
    }
}

impl Vec10 {
    pub(crate) fn new(vec: [u64; 10]) -> Self {
        Vec10(vec)
    }

    pub(crate) fn get(&self) -> [u64; 10] {
        self.0
    }

    pub(crate) fn zeros() -> Self {
        Vec10([0; 10])
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0 == [0; 10]
    }

    pub(crate) fn dot(&self, rhs: &Vec10) -> u64 {
        self.0.iter().zip(rhs.0.iter()).map(|(a, b)| a * b).sum()
    }
    
    pub fn softmax_sample(&self, temperature: f64) -> usize {
        #[allow(clippy::cast_precision_loss)]
        let scores: Vec<f64> = self.0.iter().map(|&x| x as f64).collect();
        let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = scores.iter().map(|s| {
            if *s <= 0.0 { 0.0 } else { ((s - max) / temperature).exp() }
        }).collect();
        let sum: f64 = exps.iter().sum();
        let probs: Vec<f64> = exps.iter().map(|e| e / sum).collect();

        let mut r = rand::random::<f64>();
        probs.iter().enumerate()
            .find(|(_, p)| { r -= *p; r <= 0.0 })
            .map_or(0, |(i, _)| i)
    }

    pub(crate) fn set_basic(&mut self, resources: &HashSet<BasicResourceType>) {
        self.clear_basic();
        for basic in resources {
            self.0[resource_index(ResourceType::Basic(*basic))] = 1;
        }
    }

    pub(crate) fn set_complex(&mut self, resources: &HashSet<ComplexResourceType>) {
        self.clear_complex();
        for complex in resources {
            self.0[resource_index(ResourceType::Complex(*complex))] = 1;
        }
    }

    pub(crate) fn clear_basic(&mut self) {
        self.0[0..4].fill(0);
    }

    pub(crate) fn clear_complex(&mut self) {
        self.0[4..10].fill(0);
    }

    pub(crate) fn decrease_need(&mut self, resource: &GenericResource) {
        match resource {
            GenericResource::BasicResources(basic) => {
                let idx = resource_index(ResourceType::Basic(basic.get_type()));
                self.0[idx] = self.0[idx].saturating_sub(1);
            }

            GenericResource::ComplexResources(complex) => {
                if complex.get_type() != ComplexResourceType::AIPartner {
                    let idx = resource_index(ResourceType::Complex(complex.get_type()));
                    self.0[idx] = self.0[idx].saturating_sub(1);
                }
            }
        }
    }

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
