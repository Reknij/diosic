export interface ServerInfo {
    version: string,
    author: string,
    time_running: number,
}

export interface SetupInfo {
    admin_required: boolean,
    guest_enable: boolean,
    guest_password_required: boolean,
}

export interface MediaInfo {
    id: string,
    title: string,
    album: string,
    artist: string,
    genre: string,
    year: string,
    library: string,
    cover?: string,
    categories: string[],
}

export interface MediaSourceInfo {
    title: string,
    length: number,
}


export interface SearchResult<T> {
    length: number,
    content: T[],
}

export interface UserInfo {
    username: string,
    alias: string,
    password: string,
    is_admin: boolean,
}

export interface LoginUser {
    current: UserInfo,
    token: string,
}

export interface ToSetup {
    admin: UserInfo,
    guest_enable: boolean,
    guest_password?: string,
}