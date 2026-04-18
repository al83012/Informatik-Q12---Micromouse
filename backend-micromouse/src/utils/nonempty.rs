use std::{fmt::Display, ops::Deref};


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

// pub struct NonEmptyVec<T: Sized>(Vec<T>);
//
//
// pub struct VecEmptyError;
//
//
// impl<T:Sized> TryFrom<Vec<T>> for NonEmptyVec<T> {
//     type Error = VecEmptyError;
//
//     fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
//         if value.is_empty() {
//             return Err(VecEmptyError);
//         }
//         Ok(Self(value))
//     }
// }
//
// impl<T: Sized> Into<Vec<T>> for NonEmptyVec<T> {
//     fn into(self) -> Vec<T> {
//         self.0
//     }
// }
//
//
// impl<'a, T: Sized> Into<&'a Vec<T>> for &'a NonEmptyVec<T> {
//     fn into(self) -> &'a Vec<T> {
//         &self.0
//     }
// }
//
//
// impl<T: Sized> NonEmptyVec<T> {
//     pub fn one(element: T) -> Self {
//         Self(vec![element])
//     }
// }

// impl<T: Sized> From<T> for NonEmptyVec<T> {
//     fn from(value: T) -> Self {
//         Self::one(value)
//     }
// }


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
        // f.debug_tuple("NonEmpty").field(&self.0).finish()
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
