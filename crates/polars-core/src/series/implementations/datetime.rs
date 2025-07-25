use polars_compute::rolling::QuantileMethod;

use super::*;
#[cfg(feature = "algorithm_group_by")]
use crate::frame::group_by::*;
use crate::prelude::*;

unsafe impl IntoSeries for DatetimeChunked {
    fn into_series(self) -> Series {
        Series(Arc::new(SeriesWrap(self)))
    }
}

impl private::PrivateSeriesNumeric for SeriesWrap<DatetimeChunked> {
    fn bit_repr(&self) -> Option<BitRepr> {
        Some(self.0.physical().to_bit_repr())
    }
}

impl private::PrivateSeries for SeriesWrap<DatetimeChunked> {
    fn compute_len(&mut self) {
        self.0.physical_mut().compute_len()
    }
    fn _field(&self) -> Cow<'_, Field> {
        Cow::Owned(self.0.field())
    }
    fn _dtype(&self) -> &DataType {
        self.0.dtype()
    }
    fn _get_flags(&self) -> StatisticsFlags {
        self.0.physical().get_flags()
    }
    fn _set_flags(&mut self, flags: StatisticsFlags) {
        self.0.physical_mut().set_flags(flags)
    }

    #[cfg(feature = "zip_with")]
    fn zip_with_same_type(&self, mask: &BooleanChunked, other: &Series) -> PolarsResult<Series> {
        let other = other.to_physical_repr().into_owned();
        self.0
            .physical()
            .zip_with(mask, other.as_ref().as_ref())
            .map(|ca| {
                ca.into_datetime(self.0.time_unit(), self.0.time_zone().clone())
                    .into_series()
            })
    }

    fn into_total_eq_inner<'a>(&'a self) -> Box<dyn TotalEqInner + 'a> {
        self.0.physical().into_total_eq_inner()
    }
    fn into_total_ord_inner<'a>(&'a self) -> Box<dyn TotalOrdInner + 'a> {
        self.0.physical().into_total_ord_inner()
    }

    fn vec_hash(
        &self,
        random_state: PlSeedableRandomStateQuality,
        buf: &mut Vec<u64>,
    ) -> PolarsResult<()> {
        self.0.physical().vec_hash(random_state, buf)?;
        Ok(())
    }

    fn vec_hash_combine(
        &self,
        build_hasher: PlSeedableRandomStateQuality,
        hashes: &mut [u64],
    ) -> PolarsResult<()> {
        self.0.physical().vec_hash_combine(build_hasher, hashes)?;
        Ok(())
    }

    #[cfg(feature = "algorithm_group_by")]
    unsafe fn agg_min(&self, groups: &GroupsType) -> Series {
        self.0
            .physical()
            .agg_min(groups)
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series()
    }

    #[cfg(feature = "algorithm_group_by")]
    unsafe fn agg_max(&self, groups: &GroupsType) -> Series {
        self.0
            .physical()
            .agg_max(groups)
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series()
    }
    #[cfg(feature = "algorithm_group_by")]
    unsafe fn agg_list(&self, groups: &GroupsType) -> Series {
        // we cannot cast and dispatch as the inner type of the list would be incorrect
        self.0
            .physical()
            .agg_list(groups)
            .cast(&DataType::List(Box::new(self.dtype().clone())))
            .unwrap()
    }

    fn subtract(&self, rhs: &Series) -> PolarsResult<Series> {
        match (self.dtype(), rhs.dtype()) {
            (DataType::Datetime(tu, tz), DataType::Datetime(tur, tzr)) => {
                assert_eq!(tu, tur);
                assert_eq!(tz, tzr);
                let lhs = self.cast(&DataType::Int64, CastOptions::NonStrict).unwrap();
                let rhs = rhs.cast(&DataType::Int64).unwrap();
                Ok(lhs.subtract(&rhs)?.into_duration(*tu).into_series())
            },
            (DataType::Datetime(tu, tz), DataType::Duration(tur)) => {
                assert_eq!(tu, tur);
                let lhs = self.cast(&DataType::Int64, CastOptions::NonStrict).unwrap();
                let rhs = rhs.cast(&DataType::Int64).unwrap();
                Ok(lhs
                    .subtract(&rhs)?
                    .into_datetime(*tu, tz.clone())
                    .into_series())
            },
            (dtl, dtr) => polars_bail!(opq = sub, dtl, dtr),
        }
    }
    fn add_to(&self, rhs: &Series) -> PolarsResult<Series> {
        match (self.dtype(), rhs.dtype()) {
            (DataType::Datetime(tu, tz), DataType::Duration(tur)) => {
                assert_eq!(tu, tur);
                let lhs = self.cast(&DataType::Int64, CastOptions::NonStrict).unwrap();
                let rhs = rhs.cast(&DataType::Int64).unwrap();
                Ok(lhs
                    .add_to(&rhs)?
                    .into_datetime(*tu, tz.clone())
                    .into_series())
            },
            (dtl, dtr) => polars_bail!(opq = add, dtl, dtr),
        }
    }
    fn multiply(&self, rhs: &Series) -> PolarsResult<Series> {
        polars_bail!(opq = mul, self.dtype(), rhs.dtype());
    }
    fn divide(&self, rhs: &Series) -> PolarsResult<Series> {
        polars_bail!(opq = div, self.dtype(), rhs.dtype());
    }
    fn remainder(&self, rhs: &Series) -> PolarsResult<Series> {
        polars_bail!(opq = rem, self.dtype(), rhs.dtype());
    }
    #[cfg(feature = "algorithm_group_by")]
    fn group_tuples(&self, multithreaded: bool, sorted: bool) -> PolarsResult<GroupsType> {
        self.0.physical().group_tuples(multithreaded, sorted)
    }

    fn arg_sort_multiple(
        &self,
        by: &[Column],
        options: &SortMultipleOptions,
    ) -> PolarsResult<IdxCa> {
        self.0.physical().arg_sort_multiple(by, options)
    }
}

impl SeriesTrait for SeriesWrap<DatetimeChunked> {
    fn rename(&mut self, name: PlSmallStr) {
        self.0.rename(name);
    }

    fn chunk_lengths(&self) -> ChunkLenIter<'_> {
        self.0.physical().chunk_lengths()
    }
    fn name(&self) -> &PlSmallStr {
        self.0.name()
    }

    fn chunks(&self) -> &Vec<ArrayRef> {
        self.0.physical().chunks()
    }
    unsafe fn chunks_mut(&mut self) -> &mut Vec<ArrayRef> {
        self.0.physical_mut().chunks_mut()
    }

    fn shrink_to_fit(&mut self) {
        self.0.physical_mut().shrink_to_fit()
    }

    fn slice(&self, offset: i64, length: usize) -> Series {
        self.0.slice(offset, length).into_series()
    }
    fn split_at(&self, offset: i64) -> (Series, Series) {
        let (a, b) = self.0.split_at(offset);
        (a.into_series(), b.into_series())
    }

    fn _sum_as_f64(&self) -> f64 {
        self.0.physical()._sum_as_f64()
    }

    fn mean(&self) -> Option<f64> {
        self.0.physical().mean()
    }

    fn median(&self) -> Option<f64> {
        self.0.physical().median()
    }

    fn append(&mut self, other: &Series) -> PolarsResult<()> {
        polars_ensure!(self.0.dtype() == other.dtype(), append);
        let mut other = other.to_physical_repr().into_owned();
        self.0
            .physical_mut()
            .append_owned(std::mem::take(other._get_inner_mut().as_mut()))
    }
    fn append_owned(&mut self, mut other: Series) -> PolarsResult<()> {
        polars_ensure!(self.0.dtype() == other.dtype(), append);
        self.0.physical_mut().append_owned(std::mem::take(
            &mut other
                ._get_inner_mut()
                .as_any_mut()
                .downcast_mut::<DatetimeChunked>()
                .unwrap()
                .phys,
        ))
    }

    fn extend(&mut self, other: &Series) -> PolarsResult<()> {
        polars_ensure!(self.0.dtype() == other.dtype(), extend);
        let other = other.to_physical_repr();
        self.0
            .physical_mut()
            .extend(other.as_ref().as_ref().as_ref())?;
        Ok(())
    }

    fn filter(&self, filter: &BooleanChunked) -> PolarsResult<Series> {
        self.0.physical().filter(filter).map(|ca| {
            ca.into_datetime(self.0.time_unit(), self.0.time_zone().clone())
                .into_series()
        })
    }

    fn take(&self, indices: &IdxCa) -> PolarsResult<Series> {
        let ca = self.0.physical().take(indices)?;
        Ok(ca
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series())
    }

    unsafe fn take_unchecked(&self, indices: &IdxCa) -> Series {
        let ca = self.0.physical().take_unchecked(indices);
        ca.into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series()
    }

    fn take_slice(&self, indices: &[IdxSize]) -> PolarsResult<Series> {
        let ca = self.0.physical().take(indices)?;
        Ok(ca
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series())
    }

    unsafe fn take_slice_unchecked(&self, indices: &[IdxSize]) -> Series {
        let ca = self.0.physical().take_unchecked(indices);
        ca.into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn rechunk(&self) -> Series {
        self.0
            .physical()
            .rechunk()
            .into_owned()
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series()
    }

    fn new_from_index(&self, index: usize, length: usize) -> Series {
        self.0
            .physical()
            .new_from_index(index, length)
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series()
    }

    fn cast(&self, dtype: &DataType, cast_options: CastOptions) -> PolarsResult<Series> {
        match dtype {
            DataType::String => Ok(self.0.to_string("iso")?.into_series()),
            _ => self.0.cast_with_options(dtype, cast_options),
        }
    }

    #[inline]
    unsafe fn get_unchecked(&self, index: usize) -> AnyValue<'_> {
        self.0.get_any_value_unchecked(index)
    }

    fn sort_with(&self, options: SortOptions) -> PolarsResult<Series> {
        Ok(self
            .0
            .physical()
            .sort_with(options)
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series())
    }

    fn arg_sort(&self, options: SortOptions) -> IdxCa {
        self.0.physical().arg_sort(options)
    }

    fn null_count(&self) -> usize {
        self.0.null_count()
    }

    fn has_nulls(&self) -> bool {
        self.0.has_nulls()
    }

    #[cfg(feature = "algorithm_group_by")]
    fn unique(&self) -> PolarsResult<Series> {
        self.0.physical().unique().map(|ca| {
            ca.into_datetime(self.0.time_unit(), self.0.time_zone().clone())
                .into_series()
        })
    }

    #[cfg(feature = "algorithm_group_by")]
    fn n_unique(&self) -> PolarsResult<usize> {
        self.0.physical().n_unique()
    }

    #[cfg(feature = "algorithm_group_by")]
    fn arg_unique(&self) -> PolarsResult<IdxCa> {
        self.0.physical().arg_unique()
    }

    fn is_null(&self) -> BooleanChunked {
        self.0.is_null()
    }

    fn is_not_null(&self) -> BooleanChunked {
        self.0.is_not_null()
    }

    fn reverse(&self) -> Series {
        self.0
            .physical()
            .reverse()
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series()
    }

    fn as_single_ptr(&mut self) -> PolarsResult<usize> {
        self.0.physical_mut().as_single_ptr()
    }

    fn shift(&self, periods: i64) -> Series {
        self.0
            .physical()
            .shift(periods)
            .into_datetime(self.0.time_unit(), self.0.time_zone().clone())
            .into_series()
    }

    fn max_reduce(&self) -> PolarsResult<Scalar> {
        let sc = self.0.physical().max_reduce();

        Ok(Scalar::new(self.dtype().clone(), sc.value().clone()))
    }

    fn min_reduce(&self) -> PolarsResult<Scalar> {
        let sc = self.0.physical().min_reduce();

        Ok(Scalar::new(self.dtype().clone(), sc.value().clone()))
    }

    fn median_reduce(&self) -> PolarsResult<Scalar> {
        let av: AnyValue = self.median().map(|v| v as i64).into();
        Ok(Scalar::new(self.dtype().clone(), av))
    }

    fn quantile_reduce(&self, _quantile: f64, _method: QuantileMethod) -> PolarsResult<Scalar> {
        Ok(Scalar::new(self.dtype().clone(), AnyValue::Null))
    }

    fn clone_inner(&self) -> Arc<dyn SeriesTrait> {
        Arc::new(SeriesWrap(Clone::clone(&self.0)))
    }

    fn find_validity_mismatch(&self, other: &Series, idxs: &mut Vec<IdxSize>) {
        self.0.physical().find_validity_mismatch(other, idxs)
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }

    fn as_phys_any(&self) -> &dyn Any {
        self.0.physical()
    }

    fn as_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self as _
    }
}
