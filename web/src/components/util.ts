import { ref } from "vue";
import { MediaSourceInfo, MediaInfo, UserInfo, SetupInfo } from "../models";
import { getAlbumInfo, getAlbums, getArtistInfo, getArtists, getCategories, getCategoryInfo, getGenreInfo, getGenres, getLibraries, getLibraryInfo, getYearInfo, getYears } from "../serverApi";

let current_user = ref<UserInfo>();
let setup_info = ref<SetupInfo>();

let current_media = ref<MediaInfo>()
let current_source = ref<MediaSourceInfo>()

let libraries = ref<MediaSourceInfo[]>()
let albums = ref<MediaSourceInfo[]>()
let categories = ref<MediaSourceInfo[]>()
let artists = ref<MediaSourceInfo[]>()
let genres = ref<MediaSourceInfo[]>()
let years = ref<MediaSourceInfo[]>()

async function getSourcesInfo() {
    libraries.value = await getLibraries();
    albums.value = await getAlbums();
    categories.value = await getCategories();
    artists.value = await getArtists();
    genres.value = await getGenres();
    years.value = await getYears();
}
getSourcesInfo();

let desktopMode = ref(window.innerWidth > window.innerHeight);
window.onresize = (e) => {
    desktopMode.value = window.innerWidth > window.innerHeight;
};

async function tryGetCurrentSource(query: any) {
    if (query.s && query.f) {
        switch (query.s) {
            case 'library':
                current_source.value = await getLibraryInfo(query.f, tryGetCurrentSource);
                break;
            case 'album':
                current_source.value = await getAlbumInfo(query.f, tryGetCurrentSource);
                break;
            case 'category':
                current_source.value = await getCategoryInfo(query.f, tryGetCurrentSource);
                break;
            case 'artist':
                current_source.value = await getArtistInfo(query.f, tryGetCurrentSource);
                break;
            case 'genre':
                current_source.value = await getGenreInfo(query.f, tryGetCurrentSource);
                break;
            case 'year':
                current_source.value = await getYearInfo(query.f, tryGetCurrentSource);
                break;
            default:
                current_source.value = undefined;
                break;
        }
    }
    else {
        current_source.value = undefined;
    }
}
export {
    current_user,
    setup_info,
    current_media,
    current_source,
    libraries,
    albums,
    categories,
    artists,
    genres,
    years,
    desktopMode,
    tryGetCurrentSource,
    getSourcesInfo
}