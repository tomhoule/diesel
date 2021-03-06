extern crate chrono;

pub use quickcheck::quickcheck;
use self::chrono::{NaiveDateTime, NaiveTime};

pub use schema::connection;
pub use diesel::*;
pub use diesel::result::Error;
pub use diesel::data_types::*;
pub use diesel::types::{NativeSqlType, ToSql, Nullable, Array};

use diesel::expression::AsExpression;

pub fn test_type_round_trips<ST, T>(value: T) -> bool where
    ST: NativeSqlType,
    T: AsExpression<ST> + Queryable<ST> + PartialEq + Clone + ::std::fmt::Debug,
    <T as AsExpression<ST>>::Expression: SelectableExpression<()>,
{
    let connection = connection();
    let query = select(AsExpression::<ST>::as_expression(value.clone()));
    let result = query.get_result::<T>(&connection);
    match result {
        Ok(res) => {
            if value != res {
                println!("{:?}, {:?}", value, res);
                false
            } else {
                true
            }
        }
        Err(Error::DatabaseError(msg)) =>
            &msg == "ERROR:  invalid byte sequence for encoding \"UTF8\": 0x00\n",
        Err(e) => panic!("Query failed: {:?}", e),
    }
}

pub fn id<A>(a: A) -> A { a }

macro_rules! test_round_trip {
    ($test_name:ident, $sql_type:ident, $tpe:ty) => {
        test_round_trip!($test_name, $sql_type, $tpe, id);
    };

    ($test_name:ident, $sql_type:ident, $tpe:ty, $map_fn:ident) => {
        #[test]
        fn $test_name() {
            fn round_trip(val: $tpe) -> bool {
                test_type_round_trips::<types::$sql_type, _>($map_fn(val))
            }

            fn option_round_trip(val: Option<$tpe>) -> bool {
                let val = val.map($map_fn);
                test_type_round_trips::<Nullable<types::$sql_type>, _>(val)
            }

            fn vec_round_trip(val: Vec<$tpe>) -> bool {
                let val: Vec<_> = val.into_iter().map($map_fn).collect();
                test_type_round_trips::<Array<types::$sql_type>, _>(val)
            }

            quickcheck(round_trip as fn($tpe) -> bool);
            quickcheck(option_round_trip as fn(Option<$tpe>) -> bool);
            quickcheck(vec_round_trip as fn(Vec<$tpe>) -> bool);
        }
    }
}

test_round_trip!(bool_roundtrips, Bool, bool);
test_round_trip!(i16_roundtrips, SmallInt, i16);
test_round_trip!(i32_roundtrips, Integer, i32);
test_round_trip!(i64_roundtrips, BigInt, i64);
test_round_trip!(f32_roundtrips, Float, f32);
test_round_trip!(f64_roundtrips, Double, f64);
test_round_trip!(string_roundtrips, VarChar, String);
test_round_trip!(text_roundtrips, Text, String);
test_round_trip!(binary_roundtrips, Binary, Vec<u8>);
test_round_trip!(date_roundtrips, Date, PgDate);
test_round_trip!(time_roundtrips, Time, PgTime);
test_round_trip!(timestamp_roundtrips, Timestamp, PgTimestamp);
test_round_trip!(interval_roundtrips, Interval, PgInterval);
test_round_trip!(numeric_roundtrips, Numeric, PgNumeric);
test_round_trip!(naive_datetime_roundtrips, Timestamp, (i64, u32), mk_naive_datetime);
test_round_trip!(naive_time_roundtrips, Time, (u32, u32), mk_naive_time);

fn mk_naive_datetime(data: (i64, u32)) -> NaiveDateTime {
    NaiveDateTime::from_timestamp(data.0, data.1 / 1000)
}

fn mk_naive_time(data: (u32, u32)) -> NaiveTime {
    NaiveTime::from_num_seconds_from_midnight(data.0, data.1 / 1000)
}

#[cfg(feature = "unstable")]
mod unstable_types {
    use super::*;
    use std::time::*;

    fn strip_nanosecond_precision(time: SystemTime) -> SystemTime {
        let res = match time.duration_from_earlier(UNIX_EPOCH) {
            Ok(duration) => time - Duration::new(0, duration.subsec_nanos() % 1000),
            Err(e) => time + Duration::new(0, e.duration().subsec_nanos() % 1000),
        };
        work_around_rust_lang_30173(res)
    }

    fn work_around_rust_lang_30173(time: SystemTime) -> SystemTime {
        time + Duration::new(0, 1) - Duration::new(0, 1)
    }

    test_round_trip!(systemtime_roundtrips, Timestamp, SystemTime, strip_nanosecond_precision);
}
