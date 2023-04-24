use std::time::Duration;

fn get_nminutes(d: &Duration) -> u64 {
    return d.as_secs() / 60;
}

fn get_nseconds(d: &Duration) -> u64 {
    return d.as_secs() % 60;
}

pub fn format_duration_M_S(d: &Duration) -> String {
    return format!("{}:{:0>2}", get_nminutes(d), get_nseconds(d));
}

#[test]
fn test1() {
    for i in 0..100 {
        for j in 0..59 {
            let d: Duration = Duration::from_secs(60 * i + j);
            assert_eq!(get_nminutes(&d), i);
            assert_eq!(get_nseconds(&d), j);
        }
    }
}
