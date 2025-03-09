use polars::frame::DataFrame;

pub trait IntoDataFrame {
    fn empty_data_frame() -> DataFrame;
    fn into_data_frame(&self) -> DataFrame;
}
