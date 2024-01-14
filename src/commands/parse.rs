use regex::Regex;

pub fn parse_command(input: &str) -> Command {
    let trimmed_input = input.trim();

    if let Ok(command) = parse_stop_and_routes(trimmed_input) {
        return Command::Times(command);
    }

    if let Ok(command) = parse_stops_and_location(trimmed_input) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_times_command() {
        let command = parse_command("10619 16 18");
        match command {
            Command::Times(times_command) => {
                assert_eq!(times_command.stop_number, "10619");
                assert_eq!(times_command.routes, vec!["16", "18"]);
            }
            _ => panic!("Expected TimesCommand"),
        }

        let command_with_whitespace = parse_command(" 10064 ");
        match command_with_whitespace {
            Command::Times(times_command) => {
                assert_eq!(times_command.stop_number, "10064");
                assert_eq!(times_command.routes, Vec::<String>::new());
            }
            _ => panic!("Expected TimesCommand"),
        }
    }

    #[test]
    fn test_parse_stops_command() {
        let command = parse_command("stops 245 smith");
        match command {
            Command::Stops(stops_command) => {
                assert_eq!(stops_command.location, "245 smith");
            }
            _ => panic!("Expected StopsCommand"),
        }
    }

    #[test]
    fn test_parse_unknown_command() {
        let command = parse_command("unknown command");
        match command {
            Command::Unknown(_) => (),
            _ => panic!("Expected UnknownCommand"),
        }
    }
}
