use data::LinkType;

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
#[repr(C)]
pub struct DataPart<T: LinkType + crate::TreesLinkType> {
    pub(crate) source: T,
    pub(crate) target: T,
}
