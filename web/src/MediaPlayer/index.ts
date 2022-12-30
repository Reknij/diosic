// @ts-ignore
import APlayer from 'aplayer';
import { MediaInfo } from '../models';
import { getMediaFileAddress } from '../serverApi';
import 'aplayer/dist/APlayer.min.css';
import './custom.css'
import defaultCover from '../assets/default_cover.jpg'

const aplayerElement = document.getElementById('aplayer');
const ap = new APlayer({
    container: aplayerElement,
    audio: [],
    theme: '#747474',
    fixed: true,
    listFolded: true,
});
let info = document.getElementsByClassName('aplayer-time')[0];
if (info) {
    let closebtn = document.createElement('span');
    closebtn.id = 'aplayerClose';
    closebtn.className = 'aplayer-icon';
    closebtn.innerHTML = `<svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg" data-v-029747aa=""><path fill="currentColor" d="M764.288 214.592 512 466.88 259.712 214.592a31.936 31.936 0 0 0-45.12 45.12L466.752 512 214.528 764.224a31.936 31.936 0 1 0 45.12 45.184L512 557.184l252.288 252.288a31.936 31.936 0 0 0 45.12-45.12L557.12 512.064l252.288-252.352a31.936 31.936 0 1 0-45.12-45.184z"></path></svg>`;
    closebtn.onclick = stop;
    closebtn.style.bottom = '66px';
    closebtn.style.position = 'fixed';
    closebtn.style.right = '0px';
}
let aplayerPic = document.getElementsByClassName('aplayer-pic')[0] as HTMLDivElement;
if (aplayerPic) {
    aplayerPic.style.backgroundImage = `url(${defaultCover})`;
}

try {
    window.navigator.mediaSession.setActionHandler("play", ap.toggle);
} catch (error) {
    console.log(`The media session action play is not supported yet.`)
}

try {
    window.navigator.mediaSession.setActionHandler("pause", ap.toggle);
} catch (error) {
    console.log(`The media session action pause is not supported yet.`)
}

try {
    window.navigator.mediaSession.setActionHandler("stop", function () {
        stop();
    });
} catch (error) {
    console.log(`The media session action stop is not supported yet.`)
}

try {
    window.navigator.mediaSession.setActionHandler("nexttrack", function () {
        ap.skipForward();
    });
} catch (error) {
    console.log(`The media session action nexttrack is not supported yet.`)
}

try {
    window.navigator.mediaSession.setActionHandler("previoustrack", function () {
        ap.skipBack();
    });
} catch (error) {
    console.log(`The media session action previoustrack is not supported yet.`)
}

function updateMetaData() {
    let info = getCurrentAudio().info;
    if (info) {
        document.title = `${info.title} - ${info.artist} - Diosic`;
        window.navigator.mediaSession.metadata = new MediaMetadata({
            title: info.title,
            artist: info.artist,
            album: info.album,
            artwork: [
                {
                    src:  info.cover ?? defaultCover
                }
            ],
        })
    }
}

event('canplay', updateMetaData);

function getCurrentAudio(): any {
    return ap.list.audios[ap.list.index];
}

function hide() {
    if (aplayerElement) {
        aplayerElement.style.display = 'none';
    }
}

function stop() {
    if (aplayerElement) {
        hide();
        ap.pause();
        ap.list.clear();
    }
}

function show() {
    if (aplayerElement) {
        aplayerElement.style.display = 'block';
    }
}

function event(name: string, handler: Function) {
    ap.on(name, handler);
}

async function play(media: MediaInfo) {
    ap.list.clear();
    ap.list.add([{
        name: media.title,
        artist: media.artist,
        url: await getMediaFileAddress(media.id),
        cover: media.cover ?? defaultCover,
        info: media,
    }])
    ap.play();
    show();
}

async function playList(medias: MediaInfo[]) {
    ap.list.clear();
    for (let index = 0; index < medias.length; index++) {
        const media = medias[index];
        ap.list.add([{
            name: media.title,
            artist: media.artist,
            url: await getMediaFileAddress(media.id),
            cover: media.cover ?? defaultCover,
            info: media,
        }])
    }
    ap.play();
    show();
}

export default {
    getCurrentAudio,
    play,
    playList,
    hide,
    stop,
    show,
    event
}