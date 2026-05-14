use std::cmp::PartialEq;
use std::collections::HashMap;
use std::process::Child;

#[derive(PartialEq, Eq)]
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
    process: Child,
    number: usize,
    status: Status,
    command: String,
}

impl Job{
    pub fn new(process: Child, number: usize, command: String, arguments: &Vec<String>) -> Job{
        let command = format!("{} {}", command, arguments.join(" "));
        Job{
            process,
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

    fn check_processes(&mut self){
        for job in self.active_jobs.values_mut(){
            match job.process.try_wait(){
                Ok(Some(_)) => {
                    job.status = Status::Done;
                },
                _ => {}
            }
        }
    }

    fn clean_done_jobs(&mut self){
        self.active_jobs.retain(|_, job| job.status == Status::Running);
    }

    pub fn get_all_jobs(&mut self) -> Vec<String>{
        self.check_processes();

        let mut result: Vec<String> = vec![];
        let active_jobs_len = self.active_jobs.len();
        let mut jobs = self.active_jobs.values().collect::<Vec<_>>();
        jobs.sort_by(|a, b| a.number.cmp(&b.number));
        for (index, job) in jobs.iter().enumerate(){
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
        self.clean_done_jobs();
        result
    }
}
