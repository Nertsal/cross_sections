use geng::prelude::*;

use super::vec_5::vec5;

/// 5x5 matrix
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct mat5<T>(pub(crate) [[T; 5]; 5]);

impl<T> mat5<T> {
    /// Map every element
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> mat5<U> {
        mat5(self.0.map(|row| row.map(&f)))
    }
}

impl<T: Copy> mat5<T> {
    /// Construct a matrix.
    pub fn new(values: [[T; 5]; 5]) -> Self {
        Self(values).transpose()
    }

    /// Get transposed matrix.
    pub fn transpose(self) -> Self {
        let mut result = self;
        for i in 0..5 {
            for j in 0..5 {
                result[(i, j)] = self[(j, i)];
            }
        }
        result
    }

    /// Get a row as a [vec5]
    pub fn row(&self, row_index: usize) -> vec5<T> {
        vec5(
            self[(row_index, 0)],
            self[(row_index, 1)],
            self[(row_index, 2)],
            self[(row_index, 3)],
            self[(row_index, 4)],
        )
    }

    /// Get a column as a [vec5]
    pub fn col(&self, col_index: usize) -> vec5<T> {
        vec5(
            self[(0, col_index)],
            self[(1, col_index)],
            self[(2, col_index)],
            self[(3, col_index)],
            self[(4, col_index)],
        )
    }
}

impl<T> Index<(usize, usize)> for mat5<T> {
    type Output = T;
    fn index(&self, (row, col): (usize, usize)) -> &T {
        &self.0[col][row]
    }
}

impl<T> IndexMut<(usize, usize)> for mat5<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut T {
        &mut self.0[col][row]
    }
}

impl<T> mat5<T> {
    /// Get self as a flat array
    pub fn as_flat_array(&self) -> &[T; 25] {
        unsafe { std::mem::transmute(self) }
    }
    /// Get self as a mutable flat array
    pub fn as_flat_array_mut(&mut self) -> &mut [T; 25] {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T: Num + Copy> mat5<T> {
    /// Construct zero matrix.
    pub fn zero() -> Self {
        mat5([[T::ZERO; 5]; 5])
    }

    /// Construct identity matrix.
    pub fn identity() -> Self {
        let mut result = Self::zero();
        for i in 0..5 {
            result[(i, i)] = T::ONE;
        }
        result
    }
}

impl<T: Float> Approx for mat5<T> {
    fn approx_distance_to(&self, other: &Self) -> f32 {
        let mut dist = 0.0;
        for i in 0..5 {
            for j in 0..5 {
                dist = partial_max(dist, (other[(i, j)] - self[(i, j)]).abs().as_f32());
            }
        }
        dist
    }
}
impl<T: Num + Copy> mat5<T> {
    /// Construct a uniform scale matrix.
    pub fn scale_uniform(factor: T) -> Self {
        Self::scale(vec4(factor, factor, factor, factor))
    }

    /// Construct a scale matrix.
    pub fn scale(factor: vec4<T>) -> Self {
        let mut result = Self::zero();
        result[(0, 0)] = factor.x;
        result[(1, 1)] = factor.y;
        result[(2, 2)] = factor.z;
        result[(3, 3)] = factor.w;
        result[(4, 4)] = T::ONE;
        result
    }

    /// Construct a translation matrix.
    pub fn translate(dv: vec4<T>) -> Self {
        let mut result = Self::identity();
        result[(0, 4)] = dv.x;
        result[(1, 4)] = dv.y;
        result[(2, 4)] = dv.z;
        result[(3, 4)] = dv.w;
        result
    }
}

impl<T: Float> mat5<T> {
    /// Construct matrix rotating on xy plane.
    pub fn rotate_xy(angle: Angle<T>) -> Self {
        let mut result = Self::identity();
        let (sin, cos) = angle.sin_cos();
        result[(1, 1)] = cos;
        result[(1, 2)] = -sin;
        result[(2, 1)] = sin;
        result[(2, 2)] = cos;
        result
    }

    /// Construct matrix rotating on xz plane.
    pub fn rotate_xz(angle: Angle<T>) -> Self {
        let mut result = Self::identity();
        let (sin, cos) = angle.sin_cos();
        result[(0, 0)] = cos;
        result[(2, 0)] = -sin;
        result[(0, 2)] = sin;
        result[(2, 2)] = cos;
        result
    }

    /// Construct matrix rotating on xw plane.
    pub fn rotate_xw(angle: Angle<T>) -> Self {
        let mut result = Self::identity();
        let (sin, cos) = angle.sin_cos();
        result[(0, 0)] = cos;
        result[(3, 0)] = -sin;
        result[(0, 3)] = sin;
        result[(3, 3)] = cos;
        result
    }

    /// Construct matrix rotating on yz plane.
    pub fn rotate_yz(angle: Angle<T>) -> Self {
        let mut result = Self::identity();
        let (sin, cos) = angle.sin_cos();
        result[(1, 1)] = cos;
        result[(2, 1)] = -sin;
        result[(1, 2)] = sin;
        result[(2, 2)] = cos;
        result
    }

    /// Construct matrix rotating on yw plane.
    pub fn rotate_yw(angle: Angle<T>) -> Self {
        let mut result = Self::identity();
        let (sin, cos) = angle.sin_cos();
        result[(1, 1)] = cos;
        result[(3, 1)] = -sin;
        result[(1, 3)] = sin;
        result[(3, 3)] = cos;
        result
    }

    /// Construct matrix rotating on zw plane.
    pub fn rotate_zw(angle: Angle<T>) -> Self {
        let mut result = Self::identity();
        let (sin, cos) = angle.sin_cos();
        result[(2, 2)] = cos;
        result[(3, 2)] = -sin;
        result[(2, 3)] = sin;
        result[(3, 3)] = cos;
        result
    }
}

impl<T: Num + Copy> Add for mat5<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let mut result = self;
        result += rhs;
        result
    }
}

impl<T: Num + Copy + AddAssign> AddAssign for mat5<T> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..5 {
            for j in 0..5 {
                self[(i, j)] += rhs[(i, j)];
            }
        }
    }
}

impl<T: Num + Copy> Sub for mat5<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let mut result = self;
        result -= rhs;
        result
    }
}

impl<T: Num + Copy + SubAssign> SubAssign for mat5<T> {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..5 {
            for j in 0..5 {
                self[(i, j)] -= rhs[(i, j)];
            }
        }
    }
}

impl<T: Num + Copy + Neg<Output = T>> Neg for mat5<T> {
    type Output = Self;
    fn neg(self) -> Self {
        let mut result = self;
        for i in 0..5 {
            for j in 0..5 {
                result[(i, j)] = -result[(i, j)];
            }
        }
        result
    }
}

impl<T: Num + Copy + AddAssign> Mul for mat5<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut result = mat5::new([[T::ZERO; 5]; 5]);
        for i in 0..5 {
            for j in 0..5 {
                let cur = &mut result[(i, j)];
                for t in 0..5 {
                    *cur += self[(i, t)] * rhs[(t, j)];
                }
            }
        }
        result
    }
}

impl<T: Num + Copy + AddAssign> MulAssign for mat5<T> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<T: Num + Copy> Mul<T> for mat5<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl<T: Num + Copy + MulAssign> MulAssign<T> for mat5<T> {
    fn mul_assign(&mut self, rhs: T) {
        for i in 0..5 {
            for j in 0..5 {
                self[(i, j)] *= rhs;
            }
        }
    }
}

impl<T: Num + Copy> Div<T> for mat5<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self {
        let mut result = self;
        result /= rhs;
        result
    }
}

impl<T: Num + Copy + DivAssign> DivAssign<T> for mat5<T> {
    fn div_assign(&mut self, rhs: T) {
        for i in 0..5 {
            for j in 0..5 {
                self[(i, j)] /= rhs;
            }
        }
    }
}

impl<T: Num + Copy> Mul<vec5<T>> for mat5<T> {
    type Output = vec5<T>;

    fn mul(self, rhs: vec5<T>) -> vec5<T> {
        vec5(
            vec5::dot(self.row(0), rhs),
            vec5::dot(self.row(1), rhs),
            vec5::dot(self.row(2), rhs),
            vec5::dot(self.row(3), rhs),
            vec5::dot(self.row(4), rhs),
        )
    }
}
