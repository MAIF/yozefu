use std::{fmt::Debug, fmt::Display, str::FromStr};

/// Every cluster identifier must implement this trait to be used in the command line.
pub trait Cluster: Debug + Clone + Sync + Send + 'static + FromStr + Default + Display  + FromStr<Err: Display> 
{}

impl<T> Cluster for T
where
    T: Debug + Clone + Sync + Send + 'static + FromStr + Default + Display,
    <T as FromStr>::Err: Display,
{
}
