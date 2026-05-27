use std::{fmt::Display, ops::Deref};

use serde::Serialize;


#[derive(Clone)]
pub struct NonEmpty<T: Sized>(T);


pub trait PotentiallyNonEmpty : Sized{

    fn is_empty(&self) -> bool;
    fn non_empty(self) -> Option<NonEmpty<Self>> {
        if self.is_empty() {
            None
        } else {
            Some(NonEmpty(self))
        }
    }
}


impl<T: Sized> PotentiallyNonEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T: Sized> NonEmpty<Vec<T>> {
    pub fn one(element: T) -> Self {
        Self(vec![element])
    }
}



impl<T> Display for NonEmpty<T>
    where T: Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> std::fmt::Debug for NonEmpty<T>
    where T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NonEmpty({:?})", self.0)
    }
}


impl<T> NonEmpty<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for NonEmpty<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl<T> Serialize for NonEmpty<T> 
where T: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.0.serialize(serializer)
    }
}
