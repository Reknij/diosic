export enum Source {
    Any = "any",
    Library = "library",
    Category = "category",
    Album = "album",
    Artist = "artist",
    Genre = "genre",
    Year = "year",
}

export interface UserToCreate {
    alias: string;
    username: string;
    password: string;
}

export interface UserInfo {
    id: number;
    username: string;
    password: string;
    alias: string;
    is_admin: boolean;
}

export interface ServerInfo {
    version: string;
    author: string;
    time_running: number;
    admin_required: boolean;
    guest_enable: boolean;
    guest_password_required: boolean;
}

export interface SourceInfo {
    title: string;
    total_media: number;
}

export interface PubMediaInfo {
    id: number;
    title: string;
    album: string;
    artist: string;
    genre: string;
    year: number;
    library: string;
    cover_url?: string;
    categories: string[];
    simple_rate?: number;
    bit_depth?: number;
    audio_bitrate?: number;
    overall_bitrate?: number;
    channels?: number;
    duration_seconds: number;
    file_name: string;
    file_type: string;
}

export interface ListSlice<T> {
    items: T[];
    total: number;
}

export interface GetSourcesQuery {
    limit: number;
    index: number;
    filter?: string;
}

export interface GetMediasQuery {
    limit: number;
    index: number;
    source: Source;
    filter?: string;
    to_search?: string;
}

export interface GetUsersQuery {
    limit: number;
    index: number;
    to_search?: string;
}

export interface ToSetup {
    alias: string;
    username: string;
    password: string;
    guest_enable: boolean;
    guest_password?: string;
}

export interface LoginQuery {
    username: string;
    password: string;
}

export interface LoginedResult {
    current: UserInfo; // Assuming UserInfo is defined elsewhere
    token: string;
}

export interface LogoutQuery {
    token: string;
}

export interface SearchUserQuery {
    content: string;
    index: number;
    limit: number;
}

export interface AuthQuery {
    auth: string;
}

export interface GetPluginQuery {
    index: number;
    limit: number;
}
