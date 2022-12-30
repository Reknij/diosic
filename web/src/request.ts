import axios from "axios";

let api = axios.create({
    baseURL: "/api",
    withCredentials: true,
})

export {
    api
}
