# 1. Outline
## 1.1 Introduction
A Rust backend application used as a service for managing files, much like the enterprise products: Google Drive, 
iCloud and such. The purpose is to serve as a learning project for me to master backend architecture, REST API, Rust,
automated testing and such. It is in no way intended to be used for commercial purposes.

# 2. Installation
## 2.1 Installing Rust
The recommended way of installing Rust is given in their [website](https://www.rust-lang.org/tools/install). 
The command that needs to be run is:
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
This is, however, only possible in Unix-based systems. If you are on Windows(and are not able to use Linux Subsystem 
for Windows), then follow [this link](https://forge.rust-lang.org/infra/other-installation-methods.html).

## 2.2 PostgreSQL
PostgreSQL needs to be installed for the database to run. This 
[link](https://www.enterprisedb.com/downloads/postgres-postgresql-downloads) can be followed for an
installation guide. 

## 2.3 Cloning the repo
Run the following commands to run the repo:
```shell
# Clone the repo
git clone https://github.com/nasko05/file-server-system.git
cd file-server-system
```

## 2.4 Setting environment variables
Some environment variables need to be setup in order to start the server.
```dotenv
POSTGRESQL_HOST=<value_here>
POSTGRESQL_USER=<value_here>
POSTGRESQL_PASSWORD=<value_here>
POSTGRESQL_PORT=<value_here>
POSTGRESQL_DATABASE=<value_here>
JWT_TOKEN_SECRET=<value_here>
RUST_LOG=actix_web=debug
```
These need to be put inside a `.env` file inside te `file-server-system` folder.

## 2.5 Running the application
After installing everything we need, cloning the application and setting up the environment, we are ready 
to start the app. Run these commands inside the `file-server-system` folder.
```shell
# Build the application and install all necessary dependencies
cargo build

# Start the application
cargo run
```
There might be problems with opening up a port if it is already busy. In that case one must free up port 8080.

# 3. Structure of the filesystem
The structure of the file system of the application is as follows:
- There is a centralized root_dir(that by default is created at `./root` meaning it will get created inside 
the application folder)
- Inside there is a folder for each user
- The idea is that a user can only access their own folder and noone else's
- In the future, there will be admin access and more sophisticated access policy that allows a user to have 
some group and access every folder in the group with some privilege level as well

So a user, named `test_user`, will have their own folder, named `test_user`, inside the root dir. All relative pathing
should start from there. So if a user wants to download a file in the following path: `pictures/holiday_12_2022/some_picture.png`
the path will be `pictures/holiday_12_2022` and the filename will be `some_picture.png`.

# 4. Endpoints
The workflow must always start with authentication. There is a single endpoint for authenticating the user. A **POST** 
request must be sent to `http://<host>:<port>/login` with the following body:
```json
{
    "username" : "<user>",
    "password" : "<password>"
}
```
Upon successful authentication the server should return a token, valid for 1 hour. This token should be used to 
authenticate the user as a bearer token in any further requests.
## 4.1 Uploading files
The first major endpoint is for uploading files. The endpoint expects a Multipart request and a bearer token. It then 
proceeds to upload the file in the following path: `<ROOT_DIR>/<username>/<path_from_request>`.

To upload a file send **POST** request to `/api/upload` with a multipart body. There should be two fields:
- path: Relative path to the file. Look [structure](#3-structure-of-the-filesystem) for more information
- file: the actual file

If the request is successful a status code 200 will be received, otherwise, appropriate error code and message will be 
received. Look at
[UploadFileRequest](#uploadfilerequest) for more information about the model.

## 4.2 Downloading files
The next major endpoint is for downloading files. The endpoint expects a Json body and a bearer token. It then proceeds 
to send back the file at `<ROOT_DIR>/<username>/<path_from_request>/<file_name>`, if it exists.

To download a file send a **POST** request to `/api/download/` with the following body and a bearer token:
```json
{
  "path": "<path>",
  "filename": "<filename>"
}
```
Look at [structure](#3-structure-of-the-filesystem) for more information on how to construct the path. Look at
[DownloadFileRequest](#downloadfilerequest) for more information about the model.

## 4.3 Get User Directory Structure
This endpoint is intended for use in front-end application. It returns the whole structure of a selected 
directory in a json [format](#models). It enters any subdirectories recursively.

Send a **POST** request to `/api/structure` with the following body and a bearer token:
```json
{
  "path": "<path>"
}
```
Look at [structure](#3-structure-of-the-filesystem) for more information on how to construct the path. Look at 
[FileStructureRequest](#filestructurerequest) for more information about the model.

## 4.4 Deleting User Directory
This endpoint deletes a directory inside the user directory.

Send a **POST** request to `/api/directory/delete` with the following body(check [DeleteEntityRequest](#deletingentityrequest)):
```json
{
    "path": "<path>",
    "name": "<name>"
}
```

## 4.5 Deleting User File
Similar to [4.4](#44-deleting-user-directory) the only difference is the endpoint.

Send a **POST** request to `/api/file/delete`.
## 4.6 Renaming Directory/File
To rename a directory/file the request will be:

**POST** `/directory/rename` with the following body(look at [RenameItemRequest](#renameitemrequest)):
```json
{
  "path": "<path>", 
  "old_name": "<old_name>", 
  "new_name": "<new_name>"
}
```
# Appendix
Additional information about the application
## Models
Models for all the requests.
### UploadFileRequest
```rust
pub struct UploadFileRequest {
    pub file: Option<(String, Vec<u8>)>,
    pub path: Option<String>,
}
```
### DownloadFileRequest
```rust
pub struct DownloadFileRequest {
    pub path: String,
    pub filename: String,
}
```
### FileStructureRequest
```rust
pub struct FileStructureRequest {
    pub path: String
}
```
### DeletingEntityRequest
```rust
pub struct DeleteEntityRequest {
    pub path: String,
    pub name: String,
}
```
### RenameItemRequest
```rust
pub struct RenameItemRequest {
    pub path: String,
    pub old_name: String,
    pub new_name: String
}
```