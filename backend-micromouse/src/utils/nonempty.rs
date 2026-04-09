

pub struct NonEmptyVec<T: Sized>(Vec<T>);


pub struct VecEmptyError;


impl<T:Sized> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = VecEmptyError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(VecEmptyError);
        }
        Ok(Self(value))
    }
}

impl<T: Sized> Into<Vec<T>> for NonEmptyVec<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}


impl<'a, T: Sized> Into<&'a Vec<T>> for &'a NonEmptyVec<T> {
    fn into(self) -> &'a Vec<T> {
        &self.0
    }
}


impl<T: Sized> NonEmptyVec<T> {
    pub fn one(element: T) -> Self {
        Self(vec![element])
    }
}
