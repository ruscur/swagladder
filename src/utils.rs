// Various helper functions

use time;

pub fn get_current_time() -> String {
    format!("{}", time::now_utc().rfc3339()).to_string()
}
