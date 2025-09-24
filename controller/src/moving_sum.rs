pub struct MovingSum<T, S, const N_SAMPLES: usize> {
    values: [T; N_SAMPLES],
    current_index: usize,
    total_sum: S,
    filled: usize,
}

impl<T, S, const N_SAMPLES: usize> MovingSum<T, S, N_SAMPLES>
where
    T: Copy + Default + Into<S>,
    S: Copy + Default + core::ops::SubAssign + core::ops::AddAssign,
{
    pub fn new() -> Self {
        Self {
            values: [T::default(); N_SAMPLES],
            current_index: 0,
            total_sum: S::default(),
            filled: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        let old = self.values[self.current_index];

        self.values[self.current_index] = value;

        self.total_sum -= old.into();
        self.total_sum += value.into();

        self.current_index = (self.current_index + 1) % N_SAMPLES;

        if self.filled < N_SAMPLES {
            self.filled += 1;
        }
    }

    #[inline]
    pub fn sum(&self) -> S {
        self.total_sum
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.filled
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        N_SAMPLES
    }

    pub fn average(&self) -> f32
    where
        S: Into<f32>,
    {
        if self.filled == 0 {
            return 0.0;
        }

        self.total_sum.into() / self.filled as f32
    }
}
