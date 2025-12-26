use std::collections::HashMap;

use serde::{Deserialize, Serialize, de};

use crate::legacy_json::factors::FactorScore;

pub enum Matrix {
    Scalar(f64),
    Diagonal([f64; FactorScore::NUM_TOTAL]),
    Dense(Box<[f64; FactorScore::NUM_TOTAL * FactorScore::NUM_TOTAL]>),
}

pub struct Vector([f64; FactorScore::NUM_TOTAL]);

const EPSILON: f64 = 1e-4;

impl Serialize for Matrix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Matrix::Scalar(v) => serializer.serialize_f64(*v),
            Matrix::Diagonal(arr) => arr
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, value)| value.abs() > EPSILON)
                .map(|(idx, value)| {
                    (
                        FactorScore::from_usize(idx)
                            .expect("should not be out of bound here")
                            .short_name(),
                        value,
                    )
                })
                .collect::<HashMap<_, _>>()
                .serialize(serializer),
            Matrix::Dense(arr) => arr
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, value)| value.abs() > 1e-4)
                .map(|(idx, value)| {
                    let row = FactorScore::from_usize(idx / FactorScore::NUM_TOTAL)
                        .expect("should not be out of bound here");
                    let col = FactorScore::from_usize(idx % FactorScore::NUM_TOTAL)
                        .expect("should not be out of bound here");
                    let key = if row == col {
                        row.short_name().to_string()
                    } else {
                        format!("{},{}", row.short_name(), col.short_name())
                    };
                    (key, value)
                })
                .collect::<HashMap<_, _>>()
                .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Matrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MatrixVisitor;

        impl<'de> serde::de::Visitor<'de> for MatrixVisitor {
            type Value = Matrix;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a scalar, diagonal matrix, or dense matrix")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Matrix::Scalar(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Matrix::Scalar(value as f64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Matrix::Scalar(value as f64))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut diagonal = [0.0f64; FactorScore::NUM_TOTAL];
                let mut dense = [0.0f64; FactorScore::NUM_TOTAL * FactorScore::NUM_TOTAL];
                let mut is_dense = false;

                while let Some(key) = map.next_key::<String>()? {
                    let value: f64 = map.next_value()?;
                    if key.contains(',') {
                        is_dense = true;
                        let parts: Vec<&str> = key.split(',').collect();
                        if parts.len() != 2 {
                            return Err(de::Error::custom("invalid matrix key"));
                        }
                        let row = FactorScore::from_short_name(parts[0])
                            .ok_or_else(|| de::Error::custom("invalid factor score"))?;
                        let col = FactorScore::from_short_name(parts[1])
                            .ok_or_else(|| de::Error::custom("invalid factor score"))?;
                        dense[row as usize * FactorScore::NUM_TOTAL + col as usize] = value;
                    } else {
                        let idx = FactorScore::from_short_name(&key)
                            .ok_or_else(|| de::Error::custom("invalid factor score"))?;
                        diagonal[idx as usize] = value;
                        dense[idx as usize * FactorScore::NUM_TOTAL + idx as usize] = value;
                    }
                }

                if is_dense {
                    Ok(Matrix::Dense(Box::new(dense)))
                } else {
                    Ok(Matrix::Diagonal(diagonal))
                }
            }
        }

        deserializer.deserialize_any(MatrixVisitor)
    }
}

#[test]
fn test_deserialize_scalar_matrix() {
    let json_data = "2";
    let matrix: Matrix = serde_json::from_str(json_data).unwrap();
    match matrix {
        Matrix::Scalar(v) => assert!((v - 2.0).abs() < EPSILON),
        _ => panic!("Expected Scalar matrix"),
    }
}

impl Serialize for Vector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let vec_map: HashMap<_, _> = self
            .0
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, value)| value.abs() > EPSILON)
            .map(|(idx, value)| {
                (
                    FactorScore::from_usize(idx)
                        .expect("should not be out of bound here")
                        .short_name(),
                    value,
                )
            })
            .collect();
        vec_map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Vector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VectorVisitor;

        impl<'de> serde::de::Visitor<'de> for VectorVisitor {
            type Value = Vector;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a vector represented as a map")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut vec = [0.0f64; FactorScore::NUM_TOTAL];

                while let Some(key) = map.next_key::<String>()? {
                    let value: f64 = map.next_value()?;
                    let idx = FactorScore::from_short_name(&key)
                        .ok_or_else(|| de::Error::custom("invalid factor score"))?;
                    vec[idx as usize] = value;
                }

                Ok(Vector(vec))
            }
        }

        deserializer.deserialize_map(VectorVisitor)
    }
}
