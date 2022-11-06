## Group Details

### Group Name
group-1

### Group members:
- Quan Hao NG (qhng2)
- Ian ZHANG (ianz2)
- I SUN (is16)

## Our project

### Idea
Our goal is to create our own file-sharing server (think like FTP) where we can have client devices connect to the server to upload/retrieve files. This will consist of 2 programs - one for the server and one for the client. We will run our own minimal protocol for the communication between client-server instead of implementing popular protocols.

### Motivation
We wanted to do this as we were inspired by long-time protocols such as FTP and HTTP and started off with the idea of writing a client for these popular protocols. Furthermore, we aspire to set up our own home servers / NAS storages so it would be cool to have our own tool to do access / retrieval.

## Technical Overview

### Server side program:
- Minimal-interaction CLI tool that does everything by itself, each task will be handled by a worker thread
- Tasks to implement:
1. Listening for incoming TCP connections from clients
	1. Handled with the net module
2. Setting up a thread for each client on different ports and handling incoming requests for that stream
3. Authentication, such as client logins
4. Serving up / Retrieving file

### Client side program - Commands to implement:
- *connect* - Establish a TCP connection to the server
- *login* - A simple login system 
- *mkdir* - Make a directory
- *cd* - Change directory
- *ls* - List files in current working directory
- *up* - Upload specific files
- *down* - Download specific files

## Timeline

### Checkpoint 1:
- Complete design of protocol
- Complete login and authentication features
- Complete establishment of TCP stream between server-client 
- Complete concurrent connection handling

### Checkpoint 2:
- Complete file serving / receiving feature
- Complete client side program

## Possible Challenges

### Challenge 1: Concurrency problems
- **1a: Implementing concurrent access to critical region for multiple threads** 
- Imagine a scenario where 2 separate client devices connect to the singular server and one asks for a file while the other is writing the same file. It could be as simple as one client doing a “ls” to see the files stored while that list is being updated. We need to deconflict this. This can be done with concurrency algorithms such as Peterson’s solution / Djikstra’s algorithm.
- **1b: Atomicity of operations**
- Each operation should be done to completion or not fully rolled back. An example would be if the server receives a file but the stream gets interrupted. We should not have a corrupted file in the server following that failure. 

### Challenge 2: Creating our own custom communication protocol
We plan to design our own application-layer protocol from scratch and this will come with a few challenges as listed below:
- **2a: Interpreting the input stream**
- The data that is being sent / received in the TCP streams are raw bytes. We have to come up with a scheme to interpret the incoming bytes and translate them into the specific instructions and arguments.
- **2b: Encryption**
- We can choose to implement a simple encryption scheme for our data transmitted in the TCP stream. It will be challenging to implement the encryption and decryption scheme as we are working with a custom communication protocol and given the large number of file types. 

## References
- [https://doc.rust-lang.org/std/net/struct.TcpStream.html]
