//! Tensor operations backed by ndarray.

use ndarray::Array2;
use ndarray_rand::RandomExt;
use rand::thread_rng;
use ndarray_rand::rand_distr::StandardNormal;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Add, Mul, Sub};

/// A dense tensor with f32 elements.
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub(crate) data: Array2<f32>,
}

impl Serialize for Tensor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let shape = self.shape();
        let vec = self.to_vec();
        (shape, vec).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tensor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ((rows, cols), vec): ((usize, usize), Vec<f32>) = Deserialize::deserialize(deserializer)?;
        Ok(Tensor {
            data: Array2::from_shape_vec((rows, cols), vec).unwrap(),
        })
    }
}

impl Tensor {
    /// Creates a zero tensor with given shape (rows, cols).
    pub fn zeros(shape: (usize, usize)) -> Self {
        Self {
            data: Array2::zeros(shape),
        }
    }

    /// Creates a random tensor with standard normal distribution.
    pub fn randn(shape: (usize, usize)) -> Self {
        let mut rng = thread_rng();
        Self {
            data: Array2::random_using(shape, StandardNormal, &mut rng),
        }
    }

    /// Creates a tensor filled with a constant value.
    pub fn full(shape: (usize, usize), value: f32) -> Self {
        Self {
            data: Array2::from_elem(shape, value),
        }
    }

    /// Returns the shape of the tensor.
    pub fn shape(&self) -> (usize, usize) {
        let dims = self.data.shape();
        (dims[0], dims[1])
    }

    /// Number of rows.
    pub fn nrows(&self) -> usize {
        self.data.shape()[0]
    }

    /// Number of columns.
    pub fn ncols(&self) -> usize {
        self.data.shape()[1]
    }

    /// Returns a clone of the data as a flat vector.
    pub fn to_vec(&self) -> Vec<f32> {
        self.data.iter().copied().collect()
    }

    /// Returns a clone of a specific row.
    pub fn row(&self, idx: usize) -> Tensor {
        Tensor {
            data: self.data.row(idx).to_owned().insert_axis(ndarray::Axis(0)),
        }
    }

    /// Returns a clone of a specific column.
    pub fn col(&self, idx: usize) -> Tensor {
        Tensor {
            data: self.data.column(idx).to_owned().insert_axis(ndarray::Axis(1)),
        }
    }

    /// Slices a row (returns a view as a 1-row tensor).
    pub fn slice_row(&self, idx: usize) -> Tensor {
        Tensor {
            data: self.data.slice(ndarray::s![idx, ..]).to_owned().insert_axis(ndarray::Axis(0)),
        }
    }

    /// Element-wise addition.
    pub fn add(&self, other: &Tensor) -> Tensor {
        Tensor {
            data: &self.data + &other.data,
        }
    }

    /// Element-wise subtraction.
    pub fn sub(&self, other: &Tensor) -> Tensor {
        Tensor {
            data: &self.data - &other.data,
        }
    }

    /// Element-wise multiplication.
    pub fn mul_elem(&self, other: &Tensor) -> Tensor {
        Tensor {
            data: &self.data * &other.data,
        }
    }

    /// Scalar multiplication.
    pub fn scale(&self, scalar: f32) -> Tensor {
        Tensor {
            data: &self.data * scalar,
        }
    }

    /// Matrix multiplication.
    pub fn matmul(&self, other: &Tensor) -> Tensor {
        let a = &self.data;
        let b = &other.data;
        let c = a.dot(b);
        Tensor {
            data: c,
        }
    }

    /// Element-wise map (apply function).
    pub fn mapv(&self, f: impl Fn(f32) -> f32) -> Tensor {
        Tensor {
            data: self.data.mapv(f),
        }
    }

    /// Clamp values between min and max.
    pub fn clamp(&self, min: f32, max: f32) -> Tensor {
        self.mapv(|v| v.clamp(min, max))
    }

    /// Sum along axis 0 (columns) or 1 (rows).
    pub fn sum_axis(&self, axis: usize) -> Tensor {
        let sum = if axis == 0 {
            self.data.sum_axis(ndarray::Axis(0)).insert_axis(ndarray::Axis(0))
        } else if axis == 1 {
            self.data.sum_axis(ndarray::Axis(1)).insert_axis(ndarray::Axis(1))
        } else {
            panic!("Axis must be 0 or 1");
        };
        Tensor { data: sum }
    }

    /// Mean along axis.
    pub fn mean_axis(&self, axis: usize) -> Tensor {
        let len = if axis == 0 { self.nrows() } else { self.ncols() } as f32;
        self.sum_axis(axis).scale(1.0 / len)
    }

    /// Reshape tensor to new shape. Total elements must match.
    pub fn reshape(&self, new_shape: (usize, usize)) -> Tensor {
        let new_len = new_shape.0 * new_shape.1;
        let old_len = self.data.len();
        assert_eq!(new_len, old_len, "Total elements must match");
        Tensor {
            data: self.data.clone().into_shape(new_shape).unwrap(),
        }
    }

    /// Computes the sigmoid function element-wise.
    pub fn sigmoid(&self) -> Tensor {
        self.mapv(|v| 1.0 / (1.0 + (-v).exp()))
    }

    /// Computes the element-wise exponential.
    pub fn exp(&self) -> Tensor {
        self.mapv(|v| v.exp())
    }

    /// Computes the square root of each element.
    pub fn sqrt(&self) -> Tensor {
        self.mapv(|v| v.sqrt())
    }

    /// Computes the element-wise absolute value.
    pub fn abs(&self) -> Tensor {
        self.mapv(|v| v.abs())
    }

    /// Computes the dot product between two tensors (flattened).
    pub fn dot(&self, other: &Tensor) -> f32 {
        let a = self.data.as_slice().unwrap();
        let b = other.data.as_slice().unwrap();
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Returns the maximum value in the tensor.
    pub fn max(&self) -> f32 {
        self.data.iter().copied().fold(f32::NEG_INFINITY, f32::max)
    }

    /// Returns the minimum value in the tensor.
    pub fn min(&self) -> f32 {
        self.data.iter().copied().fold(f32::INFINITY, f32::min)
    }

    /// Returns the sum of all elements.
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    pub fn transpose(&self) -> Tensor {
        Tensor {
            data: self.data.t().to_owned(),
        }
    }
}

// Operator overloads for convenience.
impl Add<&Tensor> for &Tensor {
    type Output = Tensor;
    fn add(self, other: &Tensor) -> Tensor {
        self.add(other)
    }
}

impl Add<f32> for &Tensor {
    type Output = Tensor;
    fn add(self, scalar: f32) -> Tensor {
        self.mapv(|v| v + scalar)
    }
}

impl Add<Tensor> for Tensor {
    type Output = Tensor;
    fn add(self, other: Tensor) -> Tensor {
        (&self).add(&other)
    }
}

impl Mul<&Tensor> for &Tensor {
    type Output = Tensor;
    fn mul(self, other: &Tensor) -> Tensor {
        self.mul_elem(other)
    }
}

impl Mul<f32> for &Tensor {
    type Output = Tensor;
    fn mul(self, scalar: f32) -> Tensor {
        self.scale(scalar)
    }
}

impl Sub<&Tensor> for &Tensor {
    type Output = Tensor;
    fn sub(self, other: &Tensor) -> Tensor {
        self.sub(other)
    }
}

// Conversion from ndarray to Tensor for internal use.
impl From<Array2<f32>> for Tensor {
    fn from(data: Array2<f32>) -> Self {
        Self { data }
    }
}

impl From<Tensor> for Array2<f32> {
    fn from(tensor: Tensor) -> Self {
        tensor.data
    }
}
