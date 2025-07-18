mod scalar;

#[cfg(feature = "dtype-categorical")]
mod categorical;

use std::ops::{BitAnd, Not};

use arrow::array::BooleanArray;
use arrow::bitmap::{Bitmap, BitmapBuilder};
use arrow::compute;
use num_traits::{NumCast, ToPrimitive};
use polars_compute::comparisons::{TotalEqKernel, TotalOrdKernel};

use crate::prelude::*;
use crate::series::IsSorted;
use crate::series::implementations::null::NullChunked;
use crate::utils::align_chunks_binary;

impl<T> ChunkCompareEq<&ChunkedArray<T>> for ChunkedArray<T>
where
    T: PolarsNumericType,
    T::Array: TotalOrdKernel<Scalar = T::Native> + TotalEqKernel<Scalar = T::Native>,
{
    type Item = BooleanChunked;

    fn equal(&self, rhs: &ChunkedArray<T>) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.equal(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.equal(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_eq_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn equal_missing(&self, rhs: &ChunkedArray<T>) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.equal_missing(value)
                } else {
                    self.is_null()
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.equal_missing(value)
                } else {
                    rhs.is_null()
                }
            },
            _ => arity::binary_mut_with_options(
                self,
                rhs,
                |a, b| a.tot_eq_missing_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn not_equal(&self, rhs: &ChunkedArray<T>) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.not_equal(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.not_equal(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_ne_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn not_equal_missing(&self, rhs: &ChunkedArray<T>) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.not_equal_missing(value)
                } else {
                    self.is_not_null()
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.not_equal_missing(value)
                } else {
                    rhs.is_not_null()
                }
            },
            _ => arity::binary_mut_with_options(
                self,
                rhs,
                |a, b| a.tot_ne_missing_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }
}

impl<T> ChunkCompareIneq<&ChunkedArray<T>> for ChunkedArray<T>
where
    T: PolarsNumericType,
    T::Array: TotalOrdKernel<Scalar = T::Native> + TotalEqKernel<Scalar = T::Native>,
{
    type Item = BooleanChunked;

    fn lt(&self, rhs: &ChunkedArray<T>) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.lt(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.gt(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_lt_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn lt_eq(&self, rhs: &ChunkedArray<T>) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.lt_eq(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.gt_eq(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_le_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn gt(&self, rhs: &Self) -> BooleanChunked {
        rhs.lt(self)
    }

    fn gt_eq(&self, rhs: &Self) -> BooleanChunked {
        rhs.lt_eq(self)
    }
}

impl ChunkCompareEq<&NullChunked> for NullChunked {
    type Item = BooleanChunked;

    fn equal(&self, rhs: &NullChunked) -> Self::Item {
        BooleanChunked::full_null(self.name().clone(), get_broadcast_length(self, rhs))
    }

    fn equal_missing(&self, rhs: &NullChunked) -> Self::Item {
        BooleanChunked::full(self.name().clone(), true, get_broadcast_length(self, rhs))
    }

    fn not_equal(&self, rhs: &NullChunked) -> Self::Item {
        BooleanChunked::full_null(self.name().clone(), get_broadcast_length(self, rhs))
    }

    fn not_equal_missing(&self, rhs: &NullChunked) -> Self::Item {
        BooleanChunked::full(self.name().clone(), false, get_broadcast_length(self, rhs))
    }
}

impl ChunkCompareIneq<&NullChunked> for NullChunked {
    type Item = BooleanChunked;

    fn gt(&self, rhs: &NullChunked) -> Self::Item {
        BooleanChunked::full_null(self.name().clone(), get_broadcast_length(self, rhs))
    }

    fn gt_eq(&self, rhs: &NullChunked) -> Self::Item {
        BooleanChunked::full_null(self.name().clone(), get_broadcast_length(self, rhs))
    }

    fn lt(&self, rhs: &NullChunked) -> Self::Item {
        BooleanChunked::full_null(self.name().clone(), get_broadcast_length(self, rhs))
    }

    fn lt_eq(&self, rhs: &NullChunked) -> Self::Item {
        BooleanChunked::full_null(self.name().clone(), get_broadcast_length(self, rhs))
    }
}

#[inline]
fn get_broadcast_length(lhs: &NullChunked, rhs: &NullChunked) -> usize {
    match (lhs.len(), rhs.len()) {
        (1, len_r) => len_r,
        (len_l, 1) => len_l,
        (len_l, len_r) if len_l == len_r => len_l,
        _ => panic!("Cannot compare two series of different lengths."),
    }
}

impl ChunkCompareEq<&BooleanChunked> for BooleanChunked {
    type Item = BooleanChunked;

    fn equal(&self, rhs: &BooleanChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    arity::unary_mut_values(self, |arr| arr.tot_eq_kernel_broadcast(&value).into())
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    arity::unary_mut_values(rhs, |arr| arr.tot_eq_kernel_broadcast(&value).into())
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_eq_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn equal_missing(&self, rhs: &BooleanChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    arity::unary_mut_with_options(self, |arr| {
                        arr.tot_eq_missing_kernel_broadcast(&value).into()
                    })
                } else {
                    self.is_null()
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    arity::unary_mut_with_options(rhs, |arr| {
                        arr.tot_eq_missing_kernel_broadcast(&value).into()
                    })
                } else {
                    rhs.is_null()
                }
            },
            _ => arity::binary_mut_with_options(
                self,
                rhs,
                |a, b| a.tot_eq_missing_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn not_equal(&self, rhs: &BooleanChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    arity::unary_mut_values(self, |arr| arr.tot_ne_kernel_broadcast(&value).into())
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    arity::unary_mut_values(rhs, |arr| arr.tot_ne_kernel_broadcast(&value).into())
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_ne_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn not_equal_missing(&self, rhs: &BooleanChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    arity::unary_mut_with_options(self, |arr| {
                        arr.tot_ne_missing_kernel_broadcast(&value).into()
                    })
                } else {
                    self.is_not_null()
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    arity::unary_mut_with_options(rhs, |arr| {
                        arr.tot_ne_missing_kernel_broadcast(&value).into()
                    })
                } else {
                    rhs.is_not_null()
                }
            },
            _ => arity::binary_mut_with_options(
                self,
                rhs,
                |a, b| a.tot_ne_missing_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }
}

impl ChunkCompareIneq<&BooleanChunked> for BooleanChunked {
    type Item = BooleanChunked;

    fn lt(&self, rhs: &BooleanChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    arity::unary_mut_values(self, |arr| arr.tot_lt_kernel_broadcast(&value).into())
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    arity::unary_mut_values(rhs, |arr| arr.tot_gt_kernel_broadcast(&value).into())
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_lt_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn lt_eq(&self, rhs: &BooleanChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    arity::unary_mut_values(self, |arr| arr.tot_le_kernel_broadcast(&value).into())
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    arity::unary_mut_values(rhs, |arr| arr.tot_ge_kernel_broadcast(&value).into())
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_le_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn gt(&self, rhs: &Self) -> BooleanChunked {
        rhs.lt(self)
    }

    fn gt_eq(&self, rhs: &Self) -> BooleanChunked {
        rhs.lt_eq(self)
    }
}

impl ChunkCompareEq<&StringChunked> for StringChunked {
    type Item = BooleanChunked;

    fn equal(&self, rhs: &StringChunked) -> BooleanChunked {
        self.as_binary().equal(&rhs.as_binary())
    }

    fn equal_missing(&self, rhs: &StringChunked) -> BooleanChunked {
        self.as_binary().equal_missing(&rhs.as_binary())
    }

    fn not_equal(&self, rhs: &StringChunked) -> BooleanChunked {
        self.as_binary().not_equal(&rhs.as_binary())
    }

    fn not_equal_missing(&self, rhs: &StringChunked) -> BooleanChunked {
        self.as_binary().not_equal_missing(&rhs.as_binary())
    }
}

impl ChunkCompareIneq<&StringChunked> for StringChunked {
    type Item = BooleanChunked;

    fn gt(&self, rhs: &StringChunked) -> BooleanChunked {
        self.as_binary().gt(&rhs.as_binary())
    }

    fn gt_eq(&self, rhs: &StringChunked) -> BooleanChunked {
        self.as_binary().gt_eq(&rhs.as_binary())
    }

    fn lt(&self, rhs: &StringChunked) -> BooleanChunked {
        self.as_binary().lt(&rhs.as_binary())
    }

    fn lt_eq(&self, rhs: &StringChunked) -> BooleanChunked {
        self.as_binary().lt_eq(&rhs.as_binary())
    }
}

impl ChunkCompareEq<&BinaryChunked> for BinaryChunked {
    type Item = BooleanChunked;

    fn equal(&self, rhs: &BinaryChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.equal(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.equal(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_eq_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn equal_missing(&self, rhs: &BinaryChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.equal_missing(value)
                } else {
                    self.is_null()
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.equal_missing(value)
                } else {
                    rhs.is_null()
                }
            },
            _ => arity::binary_mut_with_options(
                self,
                rhs,
                |a, b| a.tot_eq_missing_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn not_equal(&self, rhs: &BinaryChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.not_equal(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.not_equal(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_ne_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn not_equal_missing(&self, rhs: &BinaryChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.not_equal_missing(value)
                } else {
                    self.is_not_null()
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.not_equal_missing(value)
                } else {
                    rhs.is_not_null()
                }
            },
            _ => arity::binary_mut_with_options(
                self,
                rhs,
                |a, b| a.tot_ne_missing_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }
}

impl ChunkCompareIneq<&BinaryChunked> for BinaryChunked {
    type Item = BooleanChunked;

    fn lt(&self, rhs: &BinaryChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.lt(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.gt(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_lt_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn lt_eq(&self, rhs: &BinaryChunked) -> BooleanChunked {
        // Broadcast.
        match (self.len(), rhs.len()) {
            (_, 1) => {
                if let Some(value) = rhs.get(0) {
                    self.lt_eq(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, self.len())
                }
            },
            (1, _) => {
                if let Some(value) = self.get(0) {
                    rhs.gt_eq(value)
                } else {
                    BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len())
                }
            },
            _ => arity::binary_mut_values(
                self,
                rhs,
                |a, b| a.tot_le_kernel(b).into(),
                PlSmallStr::EMPTY,
            ),
        }
    }

    fn gt(&self, rhs: &Self) -> BooleanChunked {
        rhs.lt(self)
    }

    fn gt_eq(&self, rhs: &Self) -> BooleanChunked {
        rhs.lt_eq(self)
    }
}

fn _list_comparison_helper<F, B>(
    lhs: &ListChunked,
    rhs: &ListChunked,
    op: F,
    broadcast_op: B,
    missing: bool,
    is_ne: bool,
) -> BooleanChunked
where
    F: Fn(&ListArray<i64>, &ListArray<i64>) -> Bitmap,
    B: Fn(&ListArray<i64>, &Box<dyn Array>) -> Bitmap,
{
    match (lhs.len(), rhs.len()) {
        (_, 1) => {
            let right = rhs
                .downcast_iter()
                .find(|x| !x.is_empty())
                .unwrap()
                .as_any()
                .downcast_ref::<ListArray<i64>>()
                .unwrap();

            if !right.validity().is_none_or(|v| v.get(0).unwrap()) {
                if missing {
                    if is_ne {
                        return lhs.is_not_null();
                    } else {
                        return lhs.is_null();
                    }
                } else {
                    return BooleanChunked::full_null(PlSmallStr::EMPTY, lhs.len());
                }
            }

            let values = right.values().sliced(
                (*right.offsets().first()).try_into().unwrap(),
                right.offsets().range().try_into().unwrap(),
            );

            if missing {
                arity::unary_mut_with_options(lhs, |a| broadcast_op(a, &values).into())
            } else {
                arity::unary_mut_values(lhs, |a| broadcast_op(a, &values).into())
            }
        },
        (1, _) => {
            let left = lhs
                .downcast_iter()
                .find(|x| !x.is_empty())
                .unwrap()
                .as_any()
                .downcast_ref::<ListArray<i64>>()
                .unwrap();

            if !left.validity().is_none_or(|v| v.get(0).unwrap()) {
                if missing {
                    if is_ne {
                        return rhs.is_not_null();
                    } else {
                        return rhs.is_null();
                    }
                } else {
                    return BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len());
                }
            }

            let values = left.values().sliced(
                (*left.offsets().first()).try_into().unwrap(),
                left.offsets().range().try_into().unwrap(),
            );

            if missing {
                arity::unary_mut_with_options(rhs, |a| broadcast_op(a, &values).into())
            } else {
                arity::unary_mut_values(rhs, |a| broadcast_op(a, &values).into())
            }
        },
        _ => {
            if missing {
                arity::binary_mut_with_options(lhs, rhs, |a, b| op(a, b).into(), PlSmallStr::EMPTY)
            } else {
                arity::binary_mut_values(lhs, rhs, |a, b| op(a, b).into(), PlSmallStr::EMPTY)
            }
        },
    }
}

impl ChunkCompareEq<&ListChunked> for ListChunked {
    type Item = BooleanChunked;
    fn equal(&self, rhs: &ListChunked) -> BooleanChunked {
        _list_comparison_helper(
            self,
            rhs,
            TotalEqKernel::tot_eq_kernel,
            TotalEqKernel::tot_eq_kernel_broadcast,
            false,
            false,
        )
    }

    fn equal_missing(&self, rhs: &ListChunked) -> BooleanChunked {
        _list_comparison_helper(
            self,
            rhs,
            TotalEqKernel::tot_eq_missing_kernel,
            TotalEqKernel::tot_eq_missing_kernel_broadcast,
            true,
            false,
        )
    }

    fn not_equal(&self, rhs: &ListChunked) -> BooleanChunked {
        _list_comparison_helper(
            self,
            rhs,
            TotalEqKernel::tot_ne_kernel,
            TotalEqKernel::tot_ne_kernel_broadcast,
            false,
            true,
        )
    }

    fn not_equal_missing(&self, rhs: &ListChunked) -> BooleanChunked {
        _list_comparison_helper(
            self,
            rhs,
            TotalEqKernel::tot_ne_missing_kernel,
            TotalEqKernel::tot_ne_missing_kernel_broadcast,
            true,
            true,
        )
    }
}

#[cfg(feature = "dtype-struct")]
fn struct_helper<F, R>(
    a: &StructChunked,
    b: &StructChunked,
    op: F,
    reduce: R,
    op_is_ne: bool,
    is_missing: bool,
) -> BooleanChunked
where
    F: Fn(&Series, &Series) -> BooleanChunked,
    R: Fn(BooleanChunked, BooleanChunked) -> BooleanChunked,
{
    let len_a = a.len();
    let len_b = b.len();
    let broadcasts = len_a == 1 || len_b == 1;
    if (a.len() != b.len() && !broadcasts) || a.struct_fields().len() != b.struct_fields().len() {
        BooleanChunked::full(PlSmallStr::EMPTY, op_is_ne, a.len())
    } else {
        let (a, b) = align_chunks_binary(a, b);

        let mut out = a
            .fields_as_series()
            .iter()
            .zip(b.fields_as_series().iter())
            .map(|(l, r)| op(l, r))
            .reduce(&reduce)
            .unwrap_or_else(|| BooleanChunked::full(PlSmallStr::EMPTY, !op_is_ne, a.len()));

        if is_missing && (a.has_nulls() || b.has_nulls()) {
            // Do some allocations so that we can use the Series dispatch, it otherwise
            // gets complicated dealing with combinations of ==, != and broadcasting.
            let default = || {
                BooleanChunked::with_chunk(PlSmallStr::EMPTY, BooleanArray::from_slice([true]))
                    .into_series()
            };
            let validity_to_series = |x| unsafe {
                BooleanChunked::with_chunk(
                    PlSmallStr::EMPTY,
                    BooleanArray::from_inner_unchecked(ArrowDataType::Boolean, x, None),
                )
                .into_series()
            };

            out = reduce(
                out,
                op(
                    &a.rechunk_validity()
                        .map_or_else(default, validity_to_series),
                    &b.rechunk_validity()
                        .map_or_else(default, validity_to_series),
                ),
            )
        }

        if !is_missing && (a.null_count() > 0 || b.null_count() > 0) {
            let mut a = a;
            let mut b = b;

            if broadcasts {
                if a.len() == 1 {
                    a = std::borrow::Cow::Owned(a.new_from_index(0, b.len()));
                }
                if b.len() == 1 {
                    b = std::borrow::Cow::Owned(b.new_from_index(0, a.len()));
                }
            }

            let mut a = a.into_owned();
            a.zip_outer_validity(&b);
            unsafe {
                let mut new_null_count = 0;
                for (arr, a) in out.downcast_iter_mut().zip(a.downcast_iter()) {
                    arr.set_validity(a.validity().cloned());
                    new_null_count += arr.null_count();
                }
                out.set_null_count(new_null_count);
            }
        }

        out
    }
}

#[cfg(feature = "dtype-struct")]
impl ChunkCompareEq<&StructChunked> for StructChunked {
    type Item = BooleanChunked;
    fn equal(&self, rhs: &StructChunked) -> BooleanChunked {
        struct_helper(
            self,
            rhs,
            |l, r| l.equal_missing(r).unwrap(),
            |a, b| a.bitand(b),
            false,
            false,
        )
    }

    fn equal_missing(&self, rhs: &StructChunked) -> BooleanChunked {
        struct_helper(
            self,
            rhs,
            |l, r| l.equal_missing(r).unwrap(),
            |a, b| a.bitand(b),
            false,
            true,
        )
    }

    fn not_equal(&self, rhs: &StructChunked) -> BooleanChunked {
        struct_helper(
            self,
            rhs,
            |l, r| l.not_equal_missing(r).unwrap(),
            |a, b| a | b,
            true,
            false,
        )
    }

    fn not_equal_missing(&self, rhs: &StructChunked) -> BooleanChunked {
        struct_helper(
            self,
            rhs,
            |l, r| l.not_equal_missing(r).unwrap(),
            |a, b| a | b,
            true,
            true,
        )
    }
}

#[cfg(feature = "dtype-array")]
fn _array_comparison_helper<F, B>(
    lhs: &ArrayChunked,
    rhs: &ArrayChunked,
    op: F,
    broadcast_op: B,
    missing: bool,
    is_ne: bool,
) -> BooleanChunked
where
    F: Fn(&FixedSizeListArray, &FixedSizeListArray) -> Bitmap,
    B: Fn(&FixedSizeListArray, &Box<dyn Array>) -> Bitmap,
{
    match (lhs.len(), rhs.len()) {
        (_, 1) => {
            let right = rhs
                .downcast_iter()
                .find(|x| !x.is_empty())
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .unwrap();

            if !right.validity().is_none_or(|v| v.get(0).unwrap()) {
                if missing {
                    if is_ne {
                        return lhs.is_not_null();
                    } else {
                        return lhs.is_null();
                    }
                } else {
                    return BooleanChunked::full_null(PlSmallStr::EMPTY, lhs.len());
                }
            }

            if missing {
                arity::unary_mut_with_options(lhs, |a| broadcast_op(a, right.values()).into())
            } else {
                arity::unary_mut_values(lhs, |a| broadcast_op(a, right.values()).into())
            }
        },
        (1, _) => {
            let left = lhs
                .downcast_iter()
                .find(|x| !x.is_empty())
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .unwrap();

            if !left.validity().is_none_or(|v| v.get(0).unwrap()) {
                if missing {
                    if is_ne {
                        return rhs.is_not_null();
                    } else {
                        return rhs.is_null();
                    }
                } else {
                    return BooleanChunked::full_null(PlSmallStr::EMPTY, rhs.len());
                }
            }

            if missing {
                arity::unary_mut_with_options(rhs, |a| broadcast_op(a, left.values()).into())
            } else {
                arity::unary_mut_values(rhs, |a| broadcast_op(a, left.values()).into())
            }
        },
        _ => {
            if missing {
                arity::binary_mut_with_options(lhs, rhs, |a, b| op(a, b).into(), PlSmallStr::EMPTY)
            } else {
                arity::binary_mut_values(lhs, rhs, |a, b| op(a, b).into(), PlSmallStr::EMPTY)
            }
        },
    }
}

#[cfg(feature = "dtype-array")]
impl ChunkCompareEq<&ArrayChunked> for ArrayChunked {
    type Item = BooleanChunked;
    fn equal(&self, rhs: &ArrayChunked) -> BooleanChunked {
        _array_comparison_helper(
            self,
            rhs,
            TotalEqKernel::tot_eq_kernel,
            TotalEqKernel::tot_eq_kernel_broadcast,
            false,
            false,
        )
    }

    fn equal_missing(&self, rhs: &ArrayChunked) -> BooleanChunked {
        _array_comparison_helper(
            self,
            rhs,
            TotalEqKernel::tot_eq_missing_kernel,
            TotalEqKernel::tot_eq_missing_kernel_broadcast,
            true,
            false,
        )
    }

    fn not_equal(&self, rhs: &ArrayChunked) -> BooleanChunked {
        _array_comparison_helper(
            self,
            rhs,
            TotalEqKernel::tot_ne_kernel,
            TotalEqKernel::tot_ne_kernel_broadcast,
            false,
            true,
        )
    }

    fn not_equal_missing(&self, rhs: &ArrayChunked) -> Self::Item {
        _array_comparison_helper(
            self,
            rhs,
            TotalEqKernel::tot_ne_missing_kernel,
            TotalEqKernel::tot_ne_missing_kernel_broadcast,
            true,
            true,
        )
    }
}

impl Not for &BooleanChunked {
    type Output = BooleanChunked;

    fn not(self) -> Self::Output {
        let chunks = self.downcast_iter().map(compute::boolean::not);
        ChunkedArray::from_chunk_iter(self.name().clone(), chunks)
    }
}

impl Not for BooleanChunked {
    type Output = BooleanChunked;

    fn not(self) -> Self::Output {
        (&self).not()
    }
}

impl BooleanChunked {
    /// Returns whether any of the values in the column are `true`.
    ///
    /// Null values are ignored.
    pub fn any(&self) -> bool {
        self.downcast_iter().any(compute::boolean::any)
    }

    /// Returns whether all values in the array are `true`.
    ///
    /// Null values are ignored.
    pub fn all(&self) -> bool {
        self.downcast_iter().all(compute::boolean::all)
    }

    /// Returns whether any of the values in the column are `true`.
    ///
    /// The output is unknown (`None`) if the array contains any null values and
    /// no `true` values.
    pub fn any_kleene(&self) -> Option<bool> {
        let mut result = Some(false);
        for arr in self.downcast_iter() {
            match compute::boolean_kleene::any(arr) {
                Some(true) => return Some(true),
                None => result = None,
                _ => (),
            };
        }
        result
    }

    /// Returns whether all values in the column are `true`.
    ///
    /// The output is unknown (`None`) if the array contains any null values and
    /// no `false` values.
    pub fn all_kleene(&self) -> Option<bool> {
        let mut result = Some(true);
        for arr in self.downcast_iter() {
            match compute::boolean_kleene::all(arr) {
                Some(false) => return Some(false),
                None => result = None,
                _ => (),
            };
        }
        result
    }
}

// private
pub(crate) trait ChunkEqualElement {
    /// Only meant for physical types.
    /// Check if element in self is equal to element in other, assumes same dtypes
    ///
    /// # Safety
    ///
    /// No type checks.
    unsafe fn equal_element(&self, _idx_self: usize, _idx_other: usize, _other: &Series) -> bool {
        unimplemented!()
    }
}

impl<T> ChunkEqualElement for ChunkedArray<T>
where
    T: PolarsNumericType,
{
    unsafe fn equal_element(&self, idx_self: usize, idx_other: usize, other: &Series) -> bool {
        let ca_other = other.as_ref().as_ref();
        debug_assert!(self.dtype() == other.dtype());
        let ca_other = &*(ca_other as *const ChunkedArray<T>);
        // Should be get and not get_unchecked, because there could be nulls
        self.get_unchecked(idx_self)
            .tot_eq(&ca_other.get_unchecked(idx_other))
    }
}

impl ChunkEqualElement for BooleanChunked {
    unsafe fn equal_element(&self, idx_self: usize, idx_other: usize, other: &Series) -> bool {
        let ca_other = other.as_ref().as_ref();
        debug_assert!(self.dtype() == other.dtype());
        let ca_other = &*(ca_other as *const BooleanChunked);
        self.get_unchecked(idx_self) == ca_other.get_unchecked(idx_other)
    }
}

impl ChunkEqualElement for StringChunked {
    unsafe fn equal_element(&self, idx_self: usize, idx_other: usize, other: &Series) -> bool {
        let ca_other = other.as_ref().as_ref();
        debug_assert!(self.dtype() == other.dtype());
        let ca_other = &*(ca_other as *const StringChunked);
        self.get_unchecked(idx_self) == ca_other.get_unchecked(idx_other)
    }
}

impl ChunkEqualElement for BinaryChunked {
    unsafe fn equal_element(&self, idx_self: usize, idx_other: usize, other: &Series) -> bool {
        let ca_other = other.as_ref().as_ref();
        debug_assert!(self.dtype() == other.dtype());
        let ca_other = &*(ca_other as *const BinaryChunked);
        self.get_unchecked(idx_self) == ca_other.get_unchecked(idx_other)
    }
}

impl ChunkEqualElement for BinaryOffsetChunked {
    unsafe fn equal_element(&self, idx_self: usize, idx_other: usize, other: &Series) -> bool {
        let ca_other = other.as_ref().as_ref();
        debug_assert!(self.dtype() == other.dtype());
        let ca_other = &*(ca_other as *const BinaryOffsetChunked);
        self.get_unchecked(idx_self) == ca_other.get_unchecked(idx_other)
    }
}

impl ChunkEqualElement for ListChunked {}
#[cfg(feature = "dtype-array")]
impl ChunkEqualElement for ArrayChunked {}

#[cfg(test)]
#[cfg_attr(feature = "nightly", allow(clippy::manual_repeat_n))] // remove once stable
mod test {
    use std::iter::repeat_n;

    use super::super::test::get_chunked_array;
    use crate::prelude::*;

    pub(crate) fn create_two_chunked() -> (Int32Chunked, Int32Chunked) {
        let mut a1 = Int32Chunked::new(PlSmallStr::from_static("a"), &[1, 2, 3]);
        let a2 = Int32Chunked::new(PlSmallStr::from_static("a"), &[4, 5, 6]);
        let a3 = Int32Chunked::new(PlSmallStr::from_static("a"), &[1, 2, 3, 4, 5, 6]);
        a1.append(&a2).unwrap();
        (a1, a3)
    }

    #[test]
    fn test_bitwise_ops() {
        let a = BooleanChunked::new(PlSmallStr::from_static("a"), &[true, false, false]);
        let b = BooleanChunked::new(
            PlSmallStr::from_static("b"),
            &[Some(true), Some(true), None],
        );
        assert_eq!(Vec::from(&a | &b), &[Some(true), Some(true), None]);
        assert_eq!(Vec::from(&a & &b), &[Some(true), Some(false), Some(false)]);
        assert_eq!(Vec::from(!b), &[Some(false), Some(false), None]);
    }

    #[test]
    fn test_compare_chunk_diff() {
        let (a1, a2) = create_two_chunked();

        assert_eq!(
            a1.equal(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.equal(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.not_equal(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.not_equal(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.gt(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.gt(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.gt_eq(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.gt_eq(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.lt_eq(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.lt_eq(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.lt(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 6).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.lt(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_equal_chunks() {
        let a1 = get_chunked_array();
        let a2 = get_chunked_array();

        assert_eq!(
            a1.equal(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.equal(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.not_equal(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.not_equal(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.gt(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.gt(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.gt_eq(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.gt_eq(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.lt_eq(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.lt_eq(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(true), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a1.lt(&a2).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 3).collect::<Vec<_>>()
        );
        assert_eq!(
            a2.lt(&a1).into_iter().collect::<Vec<_>>(),
            repeat_n(Some(false), 3).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_null_handling() {
        // assert we comply with arrows way of handling null data
        // we check comparison on two arrays with one chunk and verify it is equal to a differently
        // chunked array comparison.

        // two same chunked arrays
        let a1: Int32Chunked = [Some(1), None, Some(3)].iter().copied().collect();
        let a2: Int32Chunked = [Some(1), Some(2), Some(3)].iter().copied().collect();

        let mut a2_2chunks: Int32Chunked = [Some(1), Some(2)].iter().copied().collect();
        a2_2chunks
            .append(&[Some(3)].iter().copied().collect())
            .unwrap();

        assert_eq!(
            a1.equal(&a2).into_iter().collect::<Vec<_>>(),
            a1.equal(&a2_2chunks).into_iter().collect::<Vec<_>>()
        );

        assert_eq!(
            a1.not_equal(&a2).into_iter().collect::<Vec<_>>(),
            a1.not_equal(&a2_2chunks).into_iter().collect::<Vec<_>>()
        );
        assert_eq!(
            a1.not_equal(&a2).into_iter().collect::<Vec<_>>(),
            a2_2chunks.not_equal(&a1).into_iter().collect::<Vec<_>>()
        );

        assert_eq!(
            a1.gt(&a2).into_iter().collect::<Vec<_>>(),
            a1.gt(&a2_2chunks).into_iter().collect::<Vec<_>>()
        );
        assert_eq!(
            a1.gt(&a2).into_iter().collect::<Vec<_>>(),
            a2_2chunks.gt(&a1).into_iter().collect::<Vec<_>>()
        );

        assert_eq!(
            a1.gt_eq(&a2).into_iter().collect::<Vec<_>>(),
            a1.gt_eq(&a2_2chunks).into_iter().collect::<Vec<_>>()
        );
        assert_eq!(
            a1.gt_eq(&a2).into_iter().collect::<Vec<_>>(),
            a2_2chunks.gt_eq(&a1).into_iter().collect::<Vec<_>>()
        );

        assert_eq!(
            a1.lt_eq(&a2).into_iter().collect::<Vec<_>>(),
            a1.lt_eq(&a2_2chunks).into_iter().collect::<Vec<_>>()
        );
        assert_eq!(
            a1.lt_eq(&a2).into_iter().collect::<Vec<_>>(),
            a2_2chunks.lt_eq(&a1).into_iter().collect::<Vec<_>>()
        );

        assert_eq!(
            a1.lt(&a2).into_iter().collect::<Vec<_>>(),
            a1.lt(&a2_2chunks).into_iter().collect::<Vec<_>>()
        );
        assert_eq!(
            a1.lt(&a2).into_iter().collect::<Vec<_>>(),
            a2_2chunks.lt(&a1).into_iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_left_right() {
        // This failed with arrow comparisons.
        // sliced
        let a1: Int32Chunked = [Some(1), Some(2)].iter().copied().collect();
        let a1 = a1.slice(1, 1);
        let a2: Int32Chunked = [Some(2)].iter().copied().collect();
        assert_eq!(a1.equal(&a2).sum(), a2.equal(&a1).sum());
        assert_eq!(a1.not_equal(&a2).sum(), a2.not_equal(&a1).sum());
        assert_eq!(a1.gt(&a2).sum(), a2.gt(&a1).sum());
        assert_eq!(a1.lt(&a2).sum(), a2.lt(&a1).sum());
        assert_eq!(a1.lt_eq(&a2).sum(), a2.lt_eq(&a1).sum());
        assert_eq!(a1.gt_eq(&a2).sum(), a2.gt_eq(&a1).sum());

        let a1: StringChunked = ["a", "b"].iter().copied().collect();
        let a1 = a1.slice(1, 1);
        let a2: StringChunked = ["b"].iter().copied().collect();
        assert_eq!(a1.equal(&a2).sum(), a2.equal(&a1).sum());
        assert_eq!(a1.not_equal(&a2).sum(), a2.not_equal(&a1).sum());
        assert_eq!(a1.gt(&a2).sum(), a2.gt(&a1).sum());
        assert_eq!(a1.lt(&a2).sum(), a2.lt(&a1).sum());
        assert_eq!(a1.lt_eq(&a2).sum(), a2.lt_eq(&a1).sum());
        assert_eq!(a1.gt_eq(&a2).sum(), a2.gt_eq(&a1).sum());
    }

    #[test]
    fn test_kleene() {
        let a = BooleanChunked::new(PlSmallStr::EMPTY, &[Some(true), Some(false), None]);
        let trues = BooleanChunked::from_slice(PlSmallStr::EMPTY, &[true, true, true]);
        let falses = BooleanChunked::from_slice(PlSmallStr::EMPTY, &[false, false, false]);

        let c = &a | &trues;
        assert_eq!(Vec::from(&c), &[Some(true), Some(true), Some(true)]);

        let c = &a | &falses;
        assert_eq!(Vec::from(&c), &[Some(true), Some(false), None])
    }

    #[test]
    fn list_broadcasting_lists() {
        let s_el = Series::new(PlSmallStr::EMPTY, &[1, 2, 3]);
        let s_lhs = Series::new(PlSmallStr::EMPTY, &[s_el.clone(), s_el.clone()]);
        let s_rhs = Series::new(PlSmallStr::EMPTY, std::slice::from_ref(&s_el));

        let result = s_lhs.list().unwrap().equal(s_rhs.list().unwrap());
        assert_eq!(result.len(), 2);
        assert!(result.all());
    }

    #[test]
    fn test_broadcasting_bools() {
        let a = BooleanChunked::from_slice(PlSmallStr::EMPTY, &[true, false, true]);
        let true_ = BooleanChunked::from_slice(PlSmallStr::EMPTY, &[true]);
        let false_ = BooleanChunked::from_slice(PlSmallStr::EMPTY, &[false]);

        let out = a.equal(&true_);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(true)]);
        let out = true_.equal(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(true)]);
        let out = a.equal(&false_);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(false)]);
        let out = false_.equal(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(false)]);

        let out = a.not_equal(&true_);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(false)]);
        let out = true_.not_equal(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(false)]);
        let out = a.not_equal(&false_);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(true)]);
        let out = false_.not_equal(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(true)]);

        let out = a.gt(&true_);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(false)]);
        let out = true_.gt(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(false)]);
        let out = a.gt(&false_);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(true)]);
        let out = false_.gt(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(false)]);

        let out = a.gt_eq(&true_);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(true)]);
        let out = true_.gt_eq(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(true)]);
        let out = a.gt_eq(&false_);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(true)]);
        let out = false_.gt_eq(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(false)]);

        let out = a.lt(&true_);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(false)]);
        let out = true_.lt(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(false)]);
        let out = a.lt(&false_);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(false)]);
        let out = false_.lt(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(true)]);

        let out = a.lt_eq(&true_);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(true)]);
        let out = true_.lt_eq(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(true)]);
        let out = a.lt_eq(&false_);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(false)]);
        let out = false_.lt_eq(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(true)]);

        let a =
            BooleanChunked::from_slice_options(PlSmallStr::EMPTY, &[Some(true), Some(false), None]);
        let all_true = BooleanChunked::from_slice(PlSmallStr::EMPTY, &[true, true, true]);
        let all_false = BooleanChunked::from_slice(PlSmallStr::EMPTY, &[false, false, false]);
        let out = a.equal(&true_);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), None]);
        let out = a.not_equal(&true_);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), None]);

        let out = a.equal(&all_true);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), None]);
        let out = a.not_equal(&all_true);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), None]);
        let out = a.equal(&false_);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), None]);
        let out = a.not_equal(&false_);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), None]);
        let out = a.equal(&all_false);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), None]);
        let out = a.not_equal(&all_false);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), None]);
    }

    #[test]
    fn test_broadcasting_numeric() {
        let a = Int32Chunked::from_slice(PlSmallStr::EMPTY, &[1, 2, 3]);
        let one = Int32Chunked::from_slice(PlSmallStr::EMPTY, &[1]);
        let three = Int32Chunked::from_slice(PlSmallStr::EMPTY, &[3]);

        let out = a.equal(&one);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(false)]);
        let out = one.equal(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(false)]);
        let out = a.equal(&three);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(true)]);
        let out = three.equal(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(true)]);

        let out = a.not_equal(&one);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(true)]);
        let out = one.not_equal(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(true)]);
        let out = a.not_equal(&three);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(false)]);
        let out = three.not_equal(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(false)]);

        let out = a.gt(&one);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(true)]);
        let out = one.gt(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(false)]);
        let out = a.gt(&three);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(false)]);
        let out = three.gt(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(false)]);

        let out = a.lt(&one);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(false)]);
        let out = one.lt(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(true), Some(true)]);
        let out = a.lt(&three);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(false)]);
        let out = three.lt(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(false)]);

        let out = a.gt_eq(&one);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(true)]);
        let out = one.gt_eq(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(false)]);
        let out = a.gt_eq(&three);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(true)]);
        let out = three.gt_eq(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(true)]);

        let out = a.lt_eq(&one);
        assert_eq!(Vec::from(&out), &[Some(true), Some(false), Some(false)]);
        let out = one.lt_eq(&a);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(true)]);
        let out = a.lt_eq(&three);
        assert_eq!(Vec::from(&out), &[Some(true), Some(true), Some(true)]);
        let out = three.lt_eq(&a);
        assert_eq!(Vec::from(&out), &[Some(false), Some(false), Some(true)]);
    }
}
