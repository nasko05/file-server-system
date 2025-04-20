# 1. Structure of the app
## Monolithic architecture
The app follows the generic architecture for online applications, 
namely the monolithic architecture.
Given the use case - side-project - and the context (designed as a simple family 
drive, where family members can store all kinds of data.) this seems appropriate.
The general structure is as follows:
- Endpoints: this is the entry-point for the app.
These classes receive http requests and strip them of their 
headers.
They then proceed to give the information to the lower layer - service - for further processing.
In the controllers or endpoints, there's no business logic.
Their only purpose is to pass info to the service layer 
and return the results, received from the service layer, appropriately, be it an error or a successful response. 
- Services: they contain all the business logic.
They receive information from the endpoints and use the data access 
layer to send requests for information from the database.
They then receive information from the database and return 
some results to the controllers.
- Data Access: this layer is tasked with managing the database.
Its sole purpose is to execute queries, since only this 
layer has access to the database.
## Project endpoints
### Authentication endpoint
This endpoint is used for getting a JSON web token - JWT - token. 
There's a simple [Data Transfer Object](#userlogin-object), DTO.
Upon authentication, the user receives a JSON web token - JWT - token that can be used in further requests.
The token lasts one hour and can't be renewed.
As a further development, a way to renew the token dynamically needs to be created, so that people
won't have to log in again to continue accessing the app.

### System operations endpoints
As the name suggests, these endpoints deal with operations on the file system.
### Delete endpoint
The endpoints for deleting a file and directory are straightforward.
A path to the parent directory is provided, the name of the user and finally the entity - file or directory - name.
For more info on how the file system is structured,
please read [Structure of the filesystem](README.md#3-structure-of-the-filesystem). 
### Download endpoint
There are two ways of downloading entities, either a file, which is downloaded as a multipart response or
an entire folder, which is first zipped and then sent as an `application/zip` response.
Again, in the request, a user must provide: their name, the path to the parent directory and the entity name.
### Directory endpoint
There's a single endpoint for creating a new empty directory, which is useful for UI applications.
It's easier to have an option for creating a directory instead of uploading it from disk.
### Get file structure endpoint
Very straightforward endpoint, used for getting the file structure of a user. 
Used primarily in the front-end for displaying purposes only.
The return format is a JSON, where each key is a parent directory, and its values are children.
For more information, please check the concrete type format of the [response](#dirtree-object).
### Rename endpoint
Although the name in the URL path suggests that this endpoint can only be used for directories, tokio has a function
that can rename an entity, which is either a directory or a file.
### Upload endpoint


# Appendix
## UserLogin object
```rust
#[derive(Debug, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}
```
## DirTree object
```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct DirTree {
    pub name: String,
    pub files: Vec<String>,
    pub dirs: Vec<DirTree>,
}
```