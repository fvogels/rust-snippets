#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cyclic<T = usize> where T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Rem<Output = T> + Copy {
    value: T,
    modulo: T,
}

impl<T> Cyclic<T> where T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Rem<Output = T> + Copy {
    pub fn new(value: T, modulo: T) -> Self {
        Cyclic { value: value % modulo, modulo }
    }

    pub fn add(&self, delta: T) -> Self {
        self.set(self.value + delta)
    }

    pub fn sub(&self, delta: T) -> Self {
        let value = self.value + self.modulo - (delta % self.modulo);

        self.set(value)
    }

    pub fn set(&self, value: T) -> Self {
        Cyclic { value: value % self.modulo, modulo: self.modulo }
    }

    pub fn value(&self) -> T {
        self.value
    }

    pub fn modulo(&self) -> T {
        self.modulo
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0, 10, 0)]
    #[case(1, 0, 10, 1)]
    #[case(1, 4, 10, 5)]
    #[case(1, 9, 10, 0)]
    #[case(1, 9, 11, 10)]
    #[case(15, 9, 20, 4)]
    #[case(15, 2, 10, 7)]
    #[case(5, 32, 10, 7)]
    fn adding(#[case] a: u64, #[case] b: u64, #[case] modulo: u64, #[case] expected: u64) {
        let a = Cyclic::new(a, modulo);
        let actual = a.add(b);

        assert_eq!(modulo, actual.modulo);
        assert_eq!(expected, actual.value);
    }

    #[rstest]
    #[case(0, 0, 10, 0)]
    #[case(1, 0, 10, 1)]
    #[case(0, 1, 10, 9)]
    #[case(0, 11, 10, 9)]
    #[case(5, 11, 10, 4)]
    #[case(9, 2, 10, 7)]
    #[case(9, 2, 5, 2)]
    #[case(9, 10, 5, 4)]
    fn subtracting(#[case] a: u64, #[case] b: u64, #[case] modulo: u64, #[case] expected: u64) {
        let a = Cyclic::new(a, modulo);
        let actual = a.sub(b);

        assert_eq!(modulo, actual.modulo);
        assert_eq!(expected, actual.value);
    }
}
