use std::collections::HashMap;

enum Status{
    Running,
    Done,
}

impl Status{
    fn string(&self) -> String{
        match self{
            Status::Running => String::from("Running"),
            Status::Done => String::from("Done"),
        }
    }
}

pub struct Job{
    pid: u32,
    number: usize,
    status: Status,
    command: String,
}

impl Job{
    pub fn new(pid: u32, number: usize, command: String, arguments: &Vec<String>) -> Job{
        let command = format!("{} {}", command, arguments.join(" "));
        Job{
            pid,
            number,
            status: Status::Running,
            command,
        }
    }

    pub fn status_string(&self) -> String{
        let total_length = 24;
        let status = self.status.string();
        let number_of_spaces = total_length - status.len();
        let mut result = String::new();
        result.push_str(&status);
        for _ in 0..number_of_spaces{
            result.push(' ');
        }
        result
    }
}

pub struct Jobs{
    active_jobs: HashMap<usize, Job>,
}

impl Jobs{
    pub fn new() -> Jobs{
        Jobs{
            active_jobs: HashMap::new(),
        }
    }

    pub fn add_job(&mut self, job: Job){
        self.active_jobs.insert(job.number, job);
    }

    pub fn get_all_jobs(&self) -> Vec<String>{
        let mut result: Vec<String> = vec![];
        let active_jobs_len = self.active_jobs.len();
        for (index, job) in self.active_jobs.values().enumerate(){
            let mut str = format!("[{}]", job.number);
            if index == active_jobs_len - 1{
                str.push_str("+");
            }
            if active_jobs_len >= 2 && index == active_jobs_len - 2{
                str.push_str("-");
            }

            str.push_str("  ");
            str.push_str(&job.status_string());
            str.push_str(&job.command);
            result.push(str);
        }
        result
    }
}
