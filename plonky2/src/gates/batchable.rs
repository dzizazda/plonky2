use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

use plonky2_field::extension_field::Extendable;

use crate::gates::gate::Gate;
use crate::hash::hash_types::RichField;
use crate::iop::target::Target;
use crate::plonk::circuit_builder::CircuitBuilder;

pub trait BatchableGate<F: RichField + Extendable<D>, const D: usize>: Gate<F, D> {
    // TODO: It would be nice to have a `Parameters` associated type.
    fn fill_gate(
        &self,
        params: &[F],
        current_slot: &CurrentSlot<F, D>,
        builder: &mut CircuitBuilder<F, D>,
    );
}

pub struct CurrentSlot<F: RichField + Extendable<D>, const D: usize> {
    current_slot: HashMap<Vec<F>, (usize, usize)>,
}

#[derive(Clone)]
pub struct GateRef<F: RichField + Extendable<D>, const D: usize>(
    pub(crate) Arc<dyn BatchableGate<F, D>>,
);

impl<F: RichField + Extendable<D>, const D: usize> GateRef<F, D> {
    pub fn new<G: BatchableGate<F, D>>(gate: G) -> GateRef<F, D> {
        GateRef(Arc::new(gate))
    }
}

impl<F: RichField + Extendable<D>, const D: usize> PartialEq for GateRef<F, D> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

impl<F: RichField + Extendable<D>, const D: usize> Hash for GateRef<F, D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.id().hash(state)
    }
}

impl<F: RichField + Extendable<D>, const D: usize> Eq for GateRef<F, D> {}

impl<F: RichField + Extendable<D>, const D: usize> Debug for GateRef<F, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0.id())
    }
}

// pub trait SingleOpGate<F: RichField + Extendable<D>, const D: usize>: Gate<F, D> {}
// impl<F: RichField + Extendable<D>, G: SingleOpGate<F, D>, const D: usize> MultiOpsGate<F, D> for G {
//     fn num_ops(&self) -> usize {
//         1
//     }
//
//     fn dependencies_ith_op(&self, gate_index: usize, i: usize) -> Vec<Target> {
//         unreachable!()
//     }
// }

pub trait MultiOpsGate<F: RichField + Extendable<D>, const D: usize>: Gate<F, D> {
    fn num_ops(&self) -> usize;

    fn dependencies_ith_op(&self, gate_index: usize, i: usize) -> Vec<Target>;
}

impl<F: RichField + Extendable<D>, G: MultiOpsGate<F, D>, const D: usize> BatchableGate<F, D>
    for G
{
    fn fill_gate(
        &self,
        params: &[F],
        current_slot: &CurrentSlot<F, D>,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        if let Some(&(gate_index, op)) = current_slot.current_slot.get(params) {
            let zero = builder.zero();
            for i in op..self.num_ops() {
                for dep in self.dependencies_ith_op(gate_index, i) {
                    builder.connect(dep, zero);
                }
            }
        }
    }
}
