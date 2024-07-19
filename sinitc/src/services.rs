use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::Path,
    process::Command,
};
use sysinfo::{Pid, Process, System};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommandExec {
    pub path: String,
    pub options: Option<Vec<String>>,
    pub environment: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Service {
    pub name: String,
    pub after: Option<Vec<String>>,
    pub exec: CommandExec,
    pub reload: Option<CommandExec>,
}

impl Service {
    pub fn pid_path(&self) -> String {
        format!("/var/run/sinitc/{}/service.pid", self.name)
    }

    pub fn pid(&self) -> u32 {
        fs::read_to_string(self.pid_path())
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .parse::<u32>()
            .unwrap()
    }

    pub fn start(&self) -> u32 {
        let command = &mut Command::new(&self.exec.path);

        if let Some(args) = &self.exec.options {
            command.args(args);
        }

        if let Some(environment) = &self.exec.environment {
            for environment_variable in environment {
                let ptr = &mut environment_variable.split('=');
                let key = ptr.next().unwrap();
                let value = ptr.next().unwrap();

                command.env(key, value);
            }
        }

        // setup logging
        // TODO: implement log rotation
        let log_dir = format!("/var/log/sinitc/{}", self.name);

        if Path::new(&log_dir).exists() {
            fs::remove_dir_all(&log_dir).unwrap();
        }

        fs::create_dir_all(&log_dir).unwrap();

        let pid = command
            .stdout(File::create(format!("{}/stdout", &log_dir)).unwrap())
            .stderr(File::create(format!("{}/stderr", &log_dir)).unwrap())
            .spawn()
            .unwrap()
            .id();

        // write pid to file
        let pid_dir = format!("/var/run/sinitc/{}", self.name);

        if Path::new(&pid_dir).exists() {
            fs::remove_dir_all(&pid_dir).unwrap();
        }

        fs::create_dir_all(&pid_dir).unwrap();

        fs::write(
            format!("{}/service.pid", pid_dir),
            pid.to_string().as_bytes(),
        )
        .unwrap();

        pid
    }

    pub fn stop(&self, process: &Process) {
        process.kill_with(sysinfo::Signal::Term);
    }

    pub fn logs(&self, log_type: &str) -> Vec<String> {
        fs::read_to_string(format!("/var/log/sinitc/{}/{}", self.name, log_type))
            .unwrap()
            .lines()
            .map(String::from)
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ServiceRoot {
    service: Service,
}

pub struct ServiceRegistry {
    pub services: Vec<Service>,
}

impl ServiceRegistry {
    pub fn discover() -> Vec<Service> {
        glob::glob("/etc/sinitc/*/service.toml")
            .unwrap()
            .map(|service_file| {
                toml::from_str(&fs::read_to_string(service_file.unwrap()).unwrap()).unwrap()
            })
            .collect::<Vec<ServiceRoot>>()
            .iter()
            .map(|service_root| service_root.service.clone())
            .collect::<Vec<Service>>()
    }

    pub fn new() -> Self {
        Self {
            services: Self::discover(),
        }
    }

    pub fn find(&self, name: &str) -> &Service {
        self.services
            .iter()
            .find(|service| service.name == name)
            .unwrap()
    }

    pub fn status_by_pid(&self, pid: u32) -> String {
        let system = System::new_all();
        let process = system.process(Pid::from_u32(pid)).unwrap();

        format!("{:?}", process.status())
    }

    pub fn status(&self, name: &str) {
        let service = self.find(name);
        let pid = service.pid();

        Self::print_line(name, &self.status_by_pid(pid));
    }

    pub fn start(&self, name: &str) {
        let service = self.find(name);
        let pid = service.start();

        ServiceRegistry::print_line(name, "Start");
        Self::print_line(name, &self.status_by_pid(pid));
    }

    pub fn stop(&self, name: &str) {
        let service = self.find(name);

        if Path::new(&service.pid_path()).exists() {
            let pid = service.pid();

            let system = System::new_all();
            let process = system.process(Pid::from_u32(pid)).unwrap();

            service.stop(process);
            Self::print_line(name, &self.status_by_pid(pid));

            fs::remove_file(service.pid_path()).unwrap();
        }
    }

    pub fn restart(&self, name: &str) {
        self.stop(name);
        self.start(name);
    }

    pub fn print_line(name: &str, line: &str) {
        println!("[sinitc] {} >>> {}", name, line);
    }

    pub fn print_log(&self, name: &str, log_type: &str) {
        let logs = self.find(name).logs(log_type);

        if !logs.is_empty() {
            println!("{}", logs.join("\n"));
        }
    }

    pub fn stdout(&self, name: &str) {
        self.print_log(name, "stdout");
    }

    pub fn stderr(&self, name: &str) {
        self.print_log(name, "stderr");
    }

    pub fn init(&self) {
        for service in &self.services {
            Command::new("/sbin/sinitc")
                .arg("start")
                .arg(&service.name)
                .spawn()
                .unwrap();
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
