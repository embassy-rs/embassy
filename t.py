import os
import re

# Regular expression to find the lines with cortex_m::asm::delay and the comment
delay_pattern = re.compile(r'cortex_m::asm::delay\((\d{1,3}(?:_\d{3})*)\);\s*//\s*At least \d{1,3}(?:_\d{3})* CPU cycles \(could be ~(\d+)\s*second[s]?\)')

def modify_line(line):
    match = delay_pattern.search(line)
    if match:
        cycles = match.group(1)
        seconds = match.group(2)
        modified_line = f"embassy_time::block_for(Duration::from_secs({seconds})); // ~{seconds} second{'s' if int(seconds) > 1 else ''}\n"
        return modified_line
    return line

def process_file(file_path):
    with open(file_path, 'r') as file:
        lines = file.readlines()
    
    modified_lines = [modify_line(line) for line in lines]

    with open(file_path, 'w') as file:
        file.writelines(modified_lines)

def find_and_process_files(root_dir):
    for dirpath, _, filenames in os.walk(root_dir):
        for filename in filenames:
            if filename.endswith('.rs'):
                file_path = os.path.join(dirpath, filename)
                process_file(file_path)

if __name__ == "__main__":
    root_directory = "."
    find_and_process_files(root_directory)
