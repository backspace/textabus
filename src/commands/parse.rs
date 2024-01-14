use regex::Regex;

pub fn parse_command(input: &str) -> Command {
    if let Ok(command) = parse_stop_and_routes(input) {
        return Command::Times(command);
    }

    if let Ok(command) = parse_stops_and_location(input) {
        return Command::Stops(command);
    }

    Command::Unknown(UnknownCommand {})
}

fn parse_stop_and_routes(input: &str) -> Result<TimesCommand, &'static str> {
    let re = Regex::new(r"^(\d{5})(?:\s+(.*))?$").unwrap();

    if let Some(captures) = re.captures(input) {
        let stop_number = captures.get(1).map_or("", |m| m.as_str()).to_string();
        let routes: Vec<String> = captures
            .get(2)
            .map_or("", |m| m.as_str())
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        Ok(TimesCommand {
            stop_number,
            routes,
        })
    } else {
        Err("Input string doesn't match the expected pattern")
    }
}

fn parse_stops_and_location(input: &str) -> Result<StopsCommand, &'static str> {
    let re = Regex::new(r"^stops\s+(.*)$").unwrap();

    if let Some(captures) = re.captures(input) {
        let location = captures.get(1).map_or("", |m| m.as_str()).to_string();
        Ok(StopsCommand { location })
    } else {
        Err("Input string does not match a stops request")
    }
}

pub enum Command {
    Times(TimesCommand),
    Stops(StopsCommand),
    Unknown(UnknownCommand),
}

pub struct TimesCommand {
    pub stop_number: String,
    pub routes: Vec<String>,
}

pub struct StopsCommand {
    pub location: String,
}

pub struct UnknownCommand;
