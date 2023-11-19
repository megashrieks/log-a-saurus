# Possible bugs
 - seems like the number of files being open does not go down fast enough even though `drop` is being called on every file open, so i've increased the ulimit to the maximum for the process
