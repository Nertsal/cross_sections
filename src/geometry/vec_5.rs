use geng::prelude::*;

/// 5 dimensional vector.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct vec5<T>(pub T, pub T, pub T, pub T, pub T);

impl<T: std::fmt::Display> std::fmt::Display for vec5<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "({}, {}, {}, {}, {})",
            self.x, self.y, self.z, self.w, self.v
        )
    }
}

impl<T> From<[T; 5]> for vec5<T> {
    fn from(arr: [T; 5]) -> vec5<T> {
        let [x, y, z, w, v] = arr;
        vec5(x, y, z, w, v)
    }
}

/// Data structure used to provide access to coordinates with the dot notation, e.g. `v.x`
#[repr(C)]
pub struct XYZWV<T> {
    #[allow(missing_docs)]
    pub x: T,
    #[allow(missing_docs)]
    pub y: T,
    #[allow(missing_docs)]
    pub z: T,
    #[allow(missing_docs)]
    pub w: T,
    #[allow(missing_docs)]
    pub v: T,
}

impl<T> Deref for XYZWV<T> {
    type Target = [T; 5];
    fn deref(&self) -> &[T; 5] {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T> DerefMut for XYZWV<T> {
    fn deref_mut(&mut self) -> &mut [T; 5] {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T> Deref for vec5<T> {
    type Target = XYZWV<T>;
    fn deref(&self) -> &XYZWV<T> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T> DerefMut for vec5<T> {
    fn deref_mut(&mut self) -> &mut XYZWV<T> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T> vec5<T> {
    /// Get first two components as a [vec2]
    pub fn xy(self) -> vec2<T> {
        vec2(self.0, self.1)
    }

    /// Get first three components as a [vec3]
    pub fn xyz(self) -> vec3<T> {
        vec3(self.0, self.1, self.2)
    }

    /// Get first four components as a [vec4]
    pub fn xyzw(self) -> vec4<T> {
        vec4(self.0, self.1, self.2, self.3)
    }

    /// Map every value (coordinate).
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> vec5<U> {
        vec5(f(self.0), f(self.1), f(self.2), f(self.3), f(self.4))
    }

    /// Zip two vectors together
    pub fn zip<U>(self, v: vec5<U>) -> vec5<(T, U)> {
        vec5(
            (self.0, v.0),
            (self.1, v.1),
            (self.2, v.2),
            (self.3, v.3),
            (self.4, v.4),
        )
    }
}

impl<T: Clone> vec5<T> {
    /// Construct a vector with all components set to specified value
    pub fn splat(value: T) -> Self {
        Self(
            value.clone(),
            value.clone(),
            value.clone(),
            value.clone(),
            value,
        )
    }
}

impl<T: UNum> vec5<T> {
    /// A zero 4-d vector
    pub const ZERO: Self = vec5(T::ZERO, T::ZERO, T::ZERO, T::ZERO, T::ZERO);

    /// A unit X
    pub const UNIT_X: Self = Self(T::ONE, T::ZERO, T::ZERO, T::ZERO, T::ZERO);

    /// A unit Y
    pub const UNIT_Y: Self = Self(T::ZERO, T::ONE, T::ZERO, T::ZERO, T::ZERO);

    /// A unit Z
    pub const UNIT_Z: Self = Self(T::ZERO, T::ZERO, T::ONE, T::ZERO, T::ZERO);

    /// A unit W
    pub const UNIT_W: Self = Self(T::ZERO, T::ZERO, T::ZERO, T::ONE, T::ZERO);

    /// A unit V
    pub const UNIT_V: Self = Self(T::ZERO, T::ZERO, T::ZERO, T::ZERO, T::ONE);
}

impl<T: Copy + Num> vec5<T> {
    /// Calculate dot product of two vectors.
    ///
    /// # Examples
    /// ```
    /// # use batbox_la::*;
    /// assert_eq!(vec5::dot(vec5(1, 2, 3, 4), vec5(3, 4, 5, 6)), 50);
    /// ```
    pub fn dot(a: Self, b: Self) -> T {
        a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w + a.v * b.v
    }
}

impl<T: Float> vec5<T> {
    /// Convert a homogenous 5d vector into 4d
    ///
    /// Same as self.xyz() / self.w
    pub fn into_4d(self) -> vec4<T> {
        self.xyzw() / self.v
    }
}

macro_rules! left_mul_impl {
    ($name:ident for $($typ:ty),*) => {$(
        impl Mul<$name<$typ>> for $typ {
            type Output = $name<$typ>;
            fn mul(self, rhs: $name<$typ>) -> $name<$typ> {
                rhs * self
            }
        }
    )*}
}

macro_rules! vec_impl_ops {
    ($name:ident : $($f:tt),*) => {
        impl<T: Add<Output=T>> Add for $name<T> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self {
                    $($f: self.$f + rhs.$f,)*
                }
            }
        }

        impl<T: AddAssign> AddAssign for $name<T> {
            fn add_assign(&mut self, rhs: Self) {
                $(self.$f += rhs.$f;)*
            }
        }

        impl<T: Sub<Output=T>> Sub for $name<T> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self {
                    $($f: self.$f - rhs.$f,)*
                }
            }
        }

        impl<T: SubAssign> SubAssign for $name<T> {
            fn sub_assign(&mut self, rhs: Self) {
                $(self.$f -= rhs.$f;)*
            }
        }

        impl<T: Neg<Output=T>> Neg for $name<T> {
            type Output = Self;
            fn neg(self) -> Self {
                Self {
                    $($f: -self.$f,)*
                }
            }
        }

        impl<T: Mul<Output=T>> Mul for $name<T> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self {
                Self {
                    $($f: self.$f * rhs.$f,)*
                }
            }
        }

        impl<T: MulAssign> MulAssign for $name<T> {
            fn mul_assign(&mut self, rhs: Self) {
                $(self.$f *= rhs.$f;)*
            }
        }

        impl<T: Div<Output=T>> Div for $name<T> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self {
                Self {
                    $($f: self.$f / rhs.$f,)*
                }
            }
        }

        impl<T: DivAssign> DivAssign for $name<T> {
            fn div_assign(&mut self, rhs: Self) {
                $(self.$f /= rhs.$f;)*
            }
        }

        impl<T: Copy + Mul<Output=T>> Mul<T> for $name<T> {
            type Output = Self;
            fn mul(self, rhs: T) -> Self {
                Self {
                    $($f: self.$f * rhs,)*
                }
            }
        }

        left_mul_impl!($name for f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, isize, usize);

        impl<T: Copy + MulAssign> MulAssign<T> for $name<T> {
            fn mul_assign(&mut self, rhs: T) {
                $(self.$f *= rhs;)*
            }
        }

        impl<T: Copy + Div<Output=T>> Div<T> for $name<T> {
            type Output = Self;
            fn div(self, rhs: T) -> Self {
                Self {
                    $($f: self.$f / rhs,)*
                }
            }
        }

        impl<T: Copy + DivAssign> DivAssign<T> for $name<T> {
            fn div_assign(&mut self, rhs: T) {
                $(self.$f /= rhs;)*
            }
        }
    };
}

vec_impl_ops!(vec5: 0, 1, 2, 3, 4);
