use std::ops::Deref;

use imageproc::point::Point;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Deserialize, Serialize)]
#[serde(remote = "Point")]
#[serde(bound(deserialize = "T: Serialize + DeserializeOwned + for<'a> Deserialize<'a>"))]
struct PointSchema<T>
where
    T: Serialize + DeserializeOwned + for<'a> Deserialize<'a>,
{
    x: T,
    y: T,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(bound(deserialize = "T: Serialize + DeserializeOwned + for<'a> Deserialize<'a>"))]
pub struct MyPoint<T: Serialize + DeserializeOwned + for<'a> Deserialize<'a>>(
    #[serde(with = "PointSchema")] Point<T>,
);

impl<T: Serialize + DeserializeOwned> From<Point<T>> for MyPoint<T> {
    fn from(value: Point<T>) -> Self {
        MyPoint(value)
    }
}

impl<T: Serialize + DeserializeOwned + Copy> From<&Point<T>> for MyPoint<T> {
    fn from(value: &Point<T>) -> Self {
        MyPoint(*value)
    }
}

impl<T: Serialize + DeserializeOwned> Deref for MyPoint<T> {
    type Target = Point<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Serialize + DeserializeOwned + Copy> Into<Point<T>> for &MyPoint<T> {
    fn into(self) -> Point<T> {
        self.0
    }
}
