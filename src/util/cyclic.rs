#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cyclic {
    value: u64,
    modulo: u64,
}

impl Cyclic {
    pub fn new(value: u64, modulo: u64) -> Self {
        assert!(modulo > 0);

        Cyclic { value: value % modulo, modulo }
    }

    pub fn add(&self, delta: u64) -> Cyclic {
        self.set(self.value + delta)
    }

    pub fn sub(&self, delta: u64) -> Cyclic {
        let value = self.value + self.modulo - (delta % self.modulo);

        self.set(value)
    }

    pub fn set(&self, value: u64) -> Cyclic {
        Cyclic { value: value % self.modulo, modulo: self.modulo }
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn modulo(&self) -> u64 {
        self.modulo
    }
}

impl From<Cyclic> for u64 {
    fn from(cyclic: Cyclic) -> Self {
        cyclic.value
    }
}

impl From<Cyclic> for usize {
    fn from(cyclic: Cyclic) -> Self {
        cyclic.value as usize
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
