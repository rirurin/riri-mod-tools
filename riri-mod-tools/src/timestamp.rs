use rkyv::{
    Archive,
    Deserialize,
    rancor::{
        Fallible,
        Source
    },
    Serialize,
    ser::Writer as RkyvWriter
};
use std::{
    collections::HashMap,
    ops::{ Deref, DerefMut },
    time::SystemTime
};

#[derive(Debug)]
pub struct WriteTime(SystemTime);
impl Archive for WriteTime {
    type Archived = [u8; size_of::<SystemTime>()];
    type Resolver = ();
    fn resolve(&self, _resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
        let val: [u8; size_of::<SystemTime>()] = unsafe { std::slice::from_raw_parts(
            &raw const self.0 as *const u8, size_of::<SystemTime>()).try_into().unwrap() };
        out.write(val);
    }
}
impl From<SystemTime> for WriteTime {
    fn from(value: SystemTime) -> Self {
        WriteTime(value)
    }
}
impl Deref for WriteTime {
    type Target = SystemTime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> Deserialize<WriteTime, D> for [u8; size_of::<SystemTime>()]
where D: Fallible + ?Sized,
      D::Error: Source
{
    fn deserialize(&self, _: &mut D) -> Result<WriteTime, <D as Fallible>::Error> {
        Ok(unsafe { std::mem::transmute::<_, _>(*self) })
    }
}

impl<S> Serialize<S> for WriteTime
where S: Fallible + RkyvWriter
{
    fn serialize(&self, _: &mut S)
        -> Result<Self::Resolver, <S as Fallible>::Error> {
        Ok(())
    }
}

#[derive(Debug)]
#[derive(Archive, Deserialize, Serialize)]
pub struct Timestamp {
    entries: HashMap<u64, WriteTime>
}
impl Deref for Timestamp {
    type Target = HashMap<u64, WriteTime>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}
impl DerefMut for Timestamp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}
impl Timestamp {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }
}