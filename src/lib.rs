use std::error::Error;
use std::process::Command;
use std::process::Output;

const IMAGE_NAME: &str = "geist";
const CONTAINER_NAME: &str = "geist";
const USERNAME: &str = "july5613";

pub async fn logs() -> Result<(), Box<dyn Error>> {
    println!("\n");
    println!("=== Geist Logs ===");
    let logs_status = Command::new("bash")
        .arg("-c")
        .arg(format!("docker logs -f {}", CONTAINER_NAME))
        .status()?;

    if !logs_status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Docker logs command failed",
        )));
    }
    Ok(())
}

async fn run_start_command(
    version: Option<String>,
    env_file_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    // Existing version string logic
    let ver = version.unwrap_or_else(|| "latest".to_string());
    let env_path = env_file_path.unwrap_or_default();

    println!("\n");
    println!("=== Starting Geist ===");
    let image_name = format!("{}/{}:{}", USERNAME, IMAGE_NAME, ver);

    // Conditionally add --env-file parameter
    let env_file_option = if !env_path.is_empty() {
        format!("--env-file {}", env_path)
    } else {
        String::new()
    };

    let run_command = format!(
        "docker run -it --rm \
        --name {} \
        --network=\"host\" \
        --env=\"DISPLAY\" \
        {} \
        --volume=\"/tmp/.X11-unix:/tmp/.X11-unix:rw\" \
        -d \
        -v /dev/bus/usb:/dev/bus/usb --device-cgroup-rule='c 189:* rmw' \
        -p 9090:9090 \
        -p 9091:9091 \
        {} \
        /bin/bash -c \"source install/setup.sh && cd src/geist && ros2 launch geist/launch/launch.py\"",
        CONTAINER_NAME,
        env_file_option, // Insert the conditional env file option here
        image_name,
    );

    // Starting the Docker container
    let start_status = Command::new("bash").arg("-c").arg(run_command).status()?;

    if !start_status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Docker run command failed",
        )));
    }

    println!("{} started successfully.", CONTAINER_NAME);
    Ok(())
}

/// This is the function that starts the Geist container
pub async fn start(
    version: Option<String>,
    env_file: Option<String>,
) -> Result<(), Box<dyn Error>> {
    // call the run_start_command function
    run_start_command(version, env_file).await?;
    Ok(())
}

/// This is the function that stops the Geist container
pub async fn stop() -> Result<(), Box<dyn Error>> {
    println!("\n");
    println!("=== Stopping Geist ===");
    let stop_status = Command::new("bash")
        .arg("-c")
        .arg(format!("docker stop {}", CONTAINER_NAME))
        .status()?;

    if !stop_status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Docker stop command failed",
        )));
    }

    println!("{} stopped successfully.", CONTAINER_NAME);
    Ok(())
}

/// This is the function that lists all services within the container
pub async fn list_services() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_docker_exec("ros2 service list")?;

    let filtered_output: Vec<String> = output
        .stdout
        .split(|c| c == &b'\n')
        .filter_map(|line| {
            let line_str = String::from_utf8_lossy(line);
            if !line_str.ends_with("/describe_parameters")
                && !line_str.ends_with("/get_parameters")
                && !line_str.ends_with("/list_parameters")
                && !line_str.ends_with("/set_parameters")
                && !line_str.ends_with("/get_parameter_types")
                && !line_str.ends_with("/set_parameters_atomically")
            {
                Some(line_str.to_string())
            } else {
                None
            }
        })
        .collect();

    // Joining the filtered lines back into a single string
    let joined_output = filtered_output.join("\n");
    println!("{}", joined_output);

    Ok(())
}

/// Makes a service call within the container
pub async fn call_service(
    address: String,
    type_: String,
    data: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let command = format!("ros2 service call {} {} \"{}\"", address, type_, data);
    let output = run_docker_exec(&command)?;
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

/// This is the function that lists all the topics within the container
pub async fn list_topics() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_docker_exec("ros2 topic list")?;

    let filtered_output: Vec<String> = output
        .stdout
        .split(|c| c == &b'\n')
        .filter_map(|line| {
            let line_str = String::from_utf8_lossy(line);
            if !line_str.contains("/client_count")
                && !line_str.contains("/connected_clients")
                && !line_str.contains("/parameter_events")
                && !line_str.contains("/rosout")
            {
                Some(line_str.to_string())
            } else {
                None
            }
        })
        .collect();

    // If you want to print the filtered output as a single string
    let joined_output = filtered_output.join("\n");
    println!("{}", joined_output);

    Ok(())
}

/// This is the function that gets the type of a topic
pub async fn get_topic_type(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let command = format!("ros2 topic type {}", name);
    let output = run_docker_exec(&command)?;
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

/// This is the function that echoes a topic
pub async fn echo_topic(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let command = format!("ros2 topic echo {}", name);
    let output = run_docker_exec(&command)?;
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

// Helper function to run `docker exec`
fn run_docker_exec(command: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let cmd = format!(
        "docker exec {} bash -c \"source install/setup.sh && {}\"",
        CONTAINER_NAME, command
    );
    let output = Command::new("bash").arg("-c").arg(cmd).output()?;

    if !output.status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Docker exec command failed",
        )));
    }

    Ok(output)
}

/// Returns the version of geist that is running
pub async fn version() -> Result<(), Box<dyn std::error::Error>> {
    // Run `cat` inside the Docker container
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "docker exec {} cat /root/geist_ws/src/geist/VERSION",
            CONTAINER_NAME
        ))
        .output()?;

    if !output.status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Docker exec command failed",
        )));
    }

    // Print the output
    println!("\n");
    println!("=== Geist Version ===");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

/// Updates the bootloader.
pub async fn update(version: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let image_path = format!("{}/{}", USERNAME, IMAGE_NAME);

    let ver = match version {
        Some(ver) => ver,
        None => "latest".to_string(),
    };

    // clean up
    println!("\n");
    println!("=== Cleaning up Geist ===");
    println!("Pruning old unused images...");
    Command::new("bash")
        .arg("-c")
        .arg("docker system prune -f")
        .status()?;
    println!("Pruning complete.");

    // update geist
    println!("\n");
    println!("=== Updating Geist ===");
    println!("Updating to version: {}", ver);

    // Add logic to update to the specified version
    let cmd = format!("docker pull {}:{}", image_path, ver);
    Command::new("bash").arg("-c").arg(cmd).status()?;

    // start the newly updated version
    run_start_command(Some(ver), None).await?;

    Ok(())
}

pub async fn status() -> Result<(), Box<dyn Error>> {
    println!("\n");
    println!("=== Geist Status ===");
    // Check if Geist is running
    let running_container_id = Command::new("docker")
        .args(["ps", "-q", "-f", &format!("name={}", CONTAINER_NAME)])
        .output()?;

    if running_container_id.stdout.is_empty() {
        println!("Geist is not running.");
    } else {
        println!("Geist is running.");

        // Show the currently running version
        let current_tag = Command::new("docker")
            .args(["inspect", "--format", "{{.Config.Image}}", CONTAINER_NAME])
            .output()?;

        if !current_tag.stdout.is_empty() {
            println!(
                "Current tag: {}",
                String::from_utf8_lossy(&current_tag.stdout).trim()
            );
        }
    }

    Ok(())
}

pub async fn exec() -> Result<(), Box<dyn Error>> {
    // Check if Geist is running
    println!("\n");
    println!("=== Connecting to Geist ===");
    let running_container_id = Command::new("docker")
        .args(["ps", "-q", "-f", &format!("name={}", CONTAINER_NAME)])
        .output()?;

    if running_container_id.stdout.is_empty() {
        println!("Geist is not currently running. Do you want to start it? [y/N]");
        let mut user_input = String::new();
        std::io::stdin().read_line(&mut user_input)?;

        if user_input.trim().eq_ignore_ascii_case("y") {
            start(None, None).await?;
        } else {
            println!("Operation canceled.");
            return Ok(());
        }
    }

    println!("Connecting to Geist. Press Ctrl+D to exit.");
    let exec_status = Command::new("docker")
        .args(["exec", "-ti", CONTAINER_NAME, "/bin/bash"])
        .status()?;

    if !exec_status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to execute an interactive shell",
        )));
    }

    Ok(())
}
