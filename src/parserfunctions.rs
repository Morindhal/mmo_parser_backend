pub mod ParserFunctions
{

    extern crate chrono;
    extern crate regex;

    use self::chrono::*;
    use self::regex::Regex;

    pub fn get_time(timestamp: &str)
        -> DateTime<UTC>
    {
        let timeparser = Regex::new(r"(?P<day_week>[A-Za-z]+) (?P<month>[A-Za-z]+)(  | )(?P<day_month>\d+) (?P<hour>\d+):(?P<minute>\d+):(?P<second>\d+) (?P<year>\d+)").unwrap();
        match timeparser.captures( timestamp ) {None => {return UTC.ymd(2016, 2, 3).and_hms(0, 0, 0);}, Some(time_cap) =>
        {
            return UTC
                                    .ymd(
                                        time_cap.name("year").unwrap().as_str().parse::<i32>().unwrap(),
                                        match time_cap.name("month").unwrap().as_str() {"Jan"=>1, "Feb"=>2, "Mar"=>3, "Apr"=>4,  "May"=>5, "Jun"=>6, "Jul"=>7, "Aug"=>8, "Sep"=>9, "Oct"=>10, "Nov"=>11, "Dec"=>12, _=>1},
                                        time_cap.name("day_month").unwrap().as_str().parse::<u32>().unwrap())
                                    .and_hms(
                                        time_cap.name("hour").unwrap().as_str().parse::<u32>().unwrap(),
                                        time_cap.name("minute").unwrap().as_str().parse::<u32>().unwrap(),
                                        time_cap.name("second").unwrap().as_str().parse::<u32>().unwrap()
                                        );
        }};
    }
}
