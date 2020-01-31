extern crate chrono;
extern crate clap;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::{App, Arg, SubCommand};
use std::fs::File;
use std::io::{self, BufRead};
use std::option::Option;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
enum RecordType {
    Header(HeaderType),
    FixRecord(FixRecord),
    Other,
}

#[derive(Debug, PartialEq, Eq)]
enum HeaderType {
    Date(NaiveDate),
    Other,
}

#[derive(Debug, PartialEq, Eq)]
struct LatLng {
    lat: i32,
    lng: i32,
}

#[derive(Debug, PartialEq, Eq)]
struct FixRecord {
    timestamp: NaiveTime,
    pos: LatLng,
    alt_baro: i32,
    alt_gps: i32,
}

fn parse_header(line: &str) -> HeaderType {
    return match &line[2..5] {
        "DTE" => {
            return HeaderType::Date(NaiveDate::parse_from_str(&line[5..11], "%d%m%y").unwrap())
        }
        _ => HeaderType::Other,
    };
}

fn parse_coordinate(hpart: usize, s: &str) -> i32 {
    let degrees = &s[0..hpart]
        .parse::<i32>()
        .expect(&("Unable to parse hh from ".to_string() + s));
    let minutes = &s[hpart..(hpart + 5)]
        .parse::<i32>()
        .expect(&("Unable to parse mm from ".to_string() + s));

    let d = &s[(hpart + 5)..(hpart + 6)];

    let mut m = minutes + degrees * 60 * 1000;
    if d == "S" || d == "E" {
        m = -m;
    }

    return m;
}

fn parse_fix(line: &str) -> FixRecord {
    let str_date = &line[1..7];
    let lat = parse_coordinate(2, &line[7..15]);
    let lng = parse_coordinate(3, &line[15..24]);
    let alt_baro = line[25..30].parse::<i32>().unwrap();
    let alt_gps = line[30..35].parse::<i32>().unwrap();

    let date = NaiveTime::parse_from_str(str_date, "%H%M%S").unwrap();

    return FixRecord {
        timestamp: date,
        pos: LatLng { lat: lat, lng: lng },
        alt_baro: alt_baro,
        alt_gps: alt_gps,
    };
}

fn parse_line(line: &str) -> RecordType {
    return match &line[0..1] {
        "H" => RecordType::Header(parse_header(line)),
        "B" => RecordType::FixRecord(parse_fix(line)),
        _ => RecordType::Other,
    };
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    let matches = App::new("igc2csv")
        .version("1.0")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let file_name = matches.value_of("INPUT").unwrap();
    let mut flight_date = None;

    println!(
        "{},{},{},{},{}",
        "time", "lat", "lng", "alt_baro", "alt_gps"
    );
    for line in read_lines(file_name).unwrap() {
        match parse_line(&line.unwrap()) {
            RecordType::Header(h) => match h {
                HeaderType::Date(d) => {
                    flight_date = Some(d);
                }
                _ => {}
            },
            RecordType::FixRecord(r) => {
                let dt = NaiveDateTime::new(flight_date.unwrap(), r.timestamp);

                println!(
                    "{},{},{},{},{}",
                    dt,
                    r.pos.lat as f32 / 60000f32,
                    r.pos.lng as f32 / 60000f32,
                    r.alt_baro,
                    r.alt_gps
                );
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_cpprdinate_0() {
        assert_eq!(parse_coordinate(3, "00000000N"), 0);
    }

    #[test]
    fn parse_coordinate_h() {
        assert_eq!(
            parse_coordinate(3, "12300000N"),
            ((123 * 60 + 0) * 1000 + 0) as i32
        );
    }

    #[test]
    fn parse_coordinate_m() {
        assert_eq!(
            parse_coordinate(3, "00012345N"),
            ((0 * 60 + 12) * 1000 + 345) as i32
        );
    }

    #[test]
    fn parse_coordinate_f() {
        assert_eq!(
            parse_coordinate(3, "12345678N"),
            ((123 * 60 + 45) * 1000 + 678) as i32
        );
    }

    #[test]
    fn parse_coordinate_f_s() {
        assert_eq!(
            parse_coordinate(3, "12345678S"),
            -((123 * 60 + 45) * 1000 + 678) as i32
        );
    }

    #[test]
    fn parse_coordinate_f_e() {
        assert_eq!(
            parse_coordinate(3, "12345678E"),
            -((123 * 60 + 45) * 1000 + 678) as i32
        );
    }

    #[test]
    fn parse_fix_1() {
        assert_eq!(
            parse_fix("B2311514647828N12025941WA0083900950"),
            FixRecord {
                timestamp: NaiveTime::from_hms(23, 11, 51),
                pos: LatLng {
                    lat: 46 * 60 * 1000 + 47828,
                    lng: 120 * 60 * 1000 + 25941
                },
                alt_baro: 839,
                alt_gps: 950
            }
        );
    }

    #[test]
    fn parse_header_1() {
        assert_eq!(
            parse_header("HFDTE161119"),
            HeaderType::Date(NaiveDate::from_ymd(2019, 11, 16))
        );
    }
}
