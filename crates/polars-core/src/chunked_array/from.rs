use super::*;

#[allow(clippy::all)]
fn from_chunks_list_dtype(chunks: &mut Vec<ArrayRef>, dtype: DataType) -> DataType {
    // ensure we don't get List<null>
    if let Some(arr) = chunks.get(0) {
        DataType::from_arrow_dtype(arr.dtype())
    } else {
        dtype
    }
}

impl<T, A> From<A> for ChunkedArray<T>
where
    T: PolarsDataType<Array = A>,
    A: Array,
{
    fn from(arr: A) -> Self {
        Self::with_chunk(PlSmallStr::EMPTY, arr)
    }
}

impl<T> ChunkedArray<T>
where
    T: PolarsDataType,
{
    pub fn with_chunk<A>(name: PlSmallStr, arr: A) -> Self
    where
        A: Array,
        T: PolarsDataType<Array = A>,
    {
        unsafe { Self::from_chunks(name, vec![Box::new(arr)]) }
    }

    pub fn with_chunk_like<A>(ca: &Self, arr: A) -> Self
    where
        A: Array,
        T: PolarsDataType<Array = A>,
    {
        Self::from_chunk_iter_like(ca, std::iter::once(arr))
    }

    pub fn from_chunk_iter<I>(name: PlSmallStr, iter: I) -> Self
    where
        I: IntoIterator,
        T: PolarsDataType<Array = <I as IntoIterator>::Item>,
        <I as IntoIterator>::Item: Array,
    {
        let chunks = iter
            .into_iter()
            .map(|x| Box::new(x) as Box<dyn Array>)
            .collect();
        unsafe { Self::from_chunks(name, chunks) }
    }

    pub fn from_chunk_iter_like<I>(ca: &Self, iter: I) -> Self
    where
        I: IntoIterator,
        T: PolarsDataType<Array = <I as IntoIterator>::Item>,
        <I as IntoIterator>::Item: Array,
    {
        let chunks = iter
            .into_iter()
            .map(|x| Box::new(x) as Box<dyn Array>)
            .collect();
        unsafe {
            Self::from_chunks_and_dtype_unchecked(ca.name().clone(), chunks, ca.dtype().clone())
        }
    }

    pub fn try_from_chunk_iter<I, A, E>(name: PlSmallStr, iter: I) -> Result<Self, E>
    where
        I: IntoIterator<Item = Result<A, E>>,
        T: PolarsDataType<Array = A>,
        A: Array,
    {
        let chunks: Result<_, _> = iter
            .into_iter()
            .map(|x| Ok(Box::new(x?) as Box<dyn Array>))
            .collect();
        unsafe { Ok(Self::from_chunks(name, chunks?)) }
    }

    pub(crate) fn from_chunk_iter_and_field<I>(field: Arc<Field>, chunks: I) -> Self
    where
        I: IntoIterator,
        T: PolarsDataType<Array = <I as IntoIterator>::Item>,
        <I as IntoIterator>::Item: Array,
    {
        assert_eq!(
            std::mem::discriminant(&T::get_static_dtype()),
            std::mem::discriminant(&field.dtype)
        );

        let mut length = 0;
        let mut null_count = 0;
        let chunks = chunks
            .into_iter()
            .map(|x| {
                length += x.len();
                null_count += x.null_count();
                Box::new(x) as Box<dyn Array>
            })
            .collect();

        unsafe { ChunkedArray::new_with_dims(field, chunks, length, null_count) }
    }

    /// Create a new [`ChunkedArray`] from existing chunks.
    ///
    /// # Safety
    /// The Arrow datatype of all chunks must match the [`PolarsDataType`] `T`.
    pub unsafe fn from_chunks(name: PlSmallStr, mut chunks: Vec<ArrayRef>) -> Self {
        let dtype = match T::get_static_dtype() {
            dtype @ DataType::List(_) => from_chunks_list_dtype(&mut chunks, dtype),
            #[cfg(feature = "dtype-array")]
            dtype @ DataType::Array(_, _) => from_chunks_list_dtype(&mut chunks, dtype),
            #[cfg(feature = "dtype-struct")]
            dtype @ DataType::Struct(_) => from_chunks_list_dtype(&mut chunks, dtype),
            dt => dt,
        };
        Self::from_chunks_and_dtype(name, chunks, dtype)
    }

    /// # Safety
    /// The Arrow datatype of all chunks must match the [`PolarsDataType`] `T`.
    pub unsafe fn with_chunks(&self, chunks: Vec<ArrayRef>) -> Self {
        ChunkedArray::new_with_compute_len(self.field.clone(), chunks)
    }

    /// Create a new [`ChunkedArray`] from existing chunks.
    ///
    /// # Safety
    ///
    /// The Arrow datatype of all chunks must match the [`PolarsDataType`] `T`.
    pub unsafe fn from_chunks_and_dtype(
        name: PlSmallStr,
        chunks: Vec<ArrayRef>,
        dtype: DataType,
    ) -> Self {
        // assertions in debug mode
        // that check if the data types in the arrays are as expected
        #[cfg(debug_assertions)]
        {
            if !chunks.is_empty() && !chunks[0].is_empty() && dtype.is_primitive() {
                assert_eq!(chunks[0].dtype(), &dtype.to_arrow(CompatLevel::newest()))
            }
        }

        Self::from_chunks_and_dtype_unchecked(name, chunks, dtype)
    }

    /// Create a new [`ChunkedArray`] from existing chunks.
    ///
    /// # Safety
    ///
    /// The Arrow datatype of all chunks must match the [`PolarsDataType`] `T`.
    pub(crate) unsafe fn from_chunks_and_dtype_unchecked(
        name: PlSmallStr,
        chunks: Vec<ArrayRef>,
        dtype: DataType,
    ) -> Self {
        let field = Arc::new(Field::new(name, dtype));
        ChunkedArray::new_with_compute_len(field, chunks)
    }

    pub fn full_null_like(ca: &Self, length: usize) -> Self {
        let chunks = std::iter::once(T::Array::full_null(
            length,
            ca.dtype().to_arrow(CompatLevel::newest()),
        ));
        Self::from_chunk_iter_like(ca, chunks)
    }
}

impl<T> ChunkedArray<T>
where
    T: PolarsNumericType,
{
    /// Create a new ChunkedArray by taking ownership of the Vec. This operation is zero copy.
    pub fn from_vec(name: PlSmallStr, v: Vec<T::Native>) -> Self {
        Self::with_chunk(name, to_primitive::<T>(v, None))
    }

    /// Create a new ChunkedArray from a Vec and a validity mask.
    pub fn from_vec_validity(
        name: PlSmallStr,
        values: Vec<T::Native>,
        buffer: Option<Bitmap>,
    ) -> Self {
        let arr = to_array::<T>(values, buffer);
        ChunkedArray::new_with_compute_len(
            Arc::new(Field::new(name, T::get_static_dtype())),
            vec![arr],
        )
    }

    /// Create a temporary [`ChunkedArray`] from a slice.
    ///
    /// # Safety
    /// The lifetime will be bound to the lifetime of the slice.
    /// This will not be checked by the borrowchecker.
    pub unsafe fn mmap_slice(name: PlSmallStr, values: &[T::Native]) -> Self {
        Self::with_chunk(name, arrow::ffi::mmap::slice(values))
    }
}

impl BooleanChunked {
    /// Create a temporary [`ChunkedArray`] from a slice.
    ///
    /// # Safety
    /// The lifetime will be bound to the lifetime of the slice.
    /// This will not be checked by the borrowchecker.
    pub unsafe fn mmap_slice(name: PlSmallStr, values: &[u8], offset: usize, len: usize) -> Self {
        let arr = arrow::ffi::mmap::bitmap(values, offset, len).unwrap();
        Self::with_chunk(name, arr)
    }

    pub fn from_bitmap(name: PlSmallStr, bitmap: Bitmap) -> Self {
        Self::with_chunk(
            name,
            BooleanArray::new(ArrowDataType::Boolean, bitmap, None),
        )
    }
}

impl<'a, T> From<&'a ChunkedArray<T>> for Vec<Option<T::Physical<'a>>>
where
    T: PolarsDataType,
{
    fn from(ca: &'a ChunkedArray<T>) -> Self {
        let mut out = Vec::with_capacity(ca.len());
        for arr in ca.downcast_iter() {
            out.extend(arr.iter())
        }
        out
    }
}
impl From<StringChunked> for Vec<Option<String>> {
    fn from(ca: StringChunked) -> Self {
        ca.iter().map(|opt| opt.map(|s| s.to_string())).collect()
    }
}

impl From<BooleanChunked> for Vec<Option<bool>> {
    fn from(ca: BooleanChunked) -> Self {
        let mut out = Vec::with_capacity(ca.len());
        for arr in ca.downcast_iter() {
            out.extend(arr.iter())
        }
        out
    }
}
