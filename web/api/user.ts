import { useApiFetch } from "./customFetch";
import type { GetUsersQuery, ListSlice, LoginQuery, LoginedResult, LogoutQuery, ToSetup, UserInfo, UserToCreate } from "./model";

export async function getUser(username: string) {
    return useApiFetch<UserInfo>(`/users/${username}`)
}

export async function getUsers(query: GetUsersQuery) {
    return useApiFetch<ListSlice<UserInfo>>(`/users`, {
        query
    })
}

export async function deleteUser(username: string) {
    return useApiFetch(`/users/${username}`, {
        watch: false,
        method: 'DELETE'
    })
}

export async function updateUser(body: UserToCreate) {
    return useApiFetch(`/users`, {
        watch: false,
        method: 'PUT',
        body
    })
}

export async function createUser(body: UserToCreate) {
    return useApiFetch(`/users`, {
        watch: false,
        method: 'POST',
        body
    })
}

export async function login(query: LoginQuery) {
    return useApiFetch<LoginedResult>(`/login`, {
        watch: false,
        query
    })
}

export async function logout(query: LogoutQuery) {
    return useApiFetch<boolean>(`/logout`, {
        watch: false,
        method: 'PUT',
        query
    })
}

export async function getCurrentUser() {
    return useApiFetch<UserInfo>(`/current_user`)
}