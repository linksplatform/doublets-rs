use doublets::{parts, split, unit, Doublets};
use mem::Global;

type LinkType = usize;
type Mem = Global<parts::LinkPart<LinkType>>;
type UnitStore = unit::Store<LinkType, Mem>;

#[test]
fn unit_type_parts() {
    let mut store = UnitStore::new(Global::new()).unwrap();
    let _ = store.create();
}

type DataMem = Global<parts::DataPart<LinkType>>;
type IndexMem = Global<parts::IndexPart<LinkType>>;
type SplitStore = split::Store<LinkType, DataMem, IndexMem>;

#[test]
fn split_type_parts() {
    let mut store = SplitStore::new(Global::new(), Global::new()).unwrap();
    let _ = store.create();
}
