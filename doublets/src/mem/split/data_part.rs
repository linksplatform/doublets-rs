use data::LinkReference;

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
#[repr(C)]
pub struct DataPart<T: LinkReference> {
    pub(crate) source: T,
    pub(crate) target: T,
}
