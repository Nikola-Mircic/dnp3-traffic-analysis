# DNP3 traffic analysis
Project for my bachelor's thesis

## Rust setup
  - Clone the `main` branch of this repository: 
    ```bash
    gh repo clone Nikola-Mircic/dnp3-traffic-analysis
    ```
    
  - Run the **outstation** simulator:
    ```bash
    cargo run -p outstation
    ```
    
  - Run the **master** simulator:
    ```bash
    cargo run -p master
    ```
    
  - Run the **capture** program:
    ```bash
    cargo run -p capture
    ```

## Python setup ( Docker )
  - Clone the `python` branch of this repository:
    ```bash
    gh repo clone Nikola-Mircic/dnp3-traffic-analysis
    git checkout python
    ```
  - Run with:
    ```bash
    docker compose up
    ```
    It will start 5 Docker containers:
      - *master* - container running master simulator
      - *capture* - container capturing traffic coming in and out of *master* container
      - *outstation-1*/*outstation-2*/*outstation-3* - three containers, each running separate outstation simulator
