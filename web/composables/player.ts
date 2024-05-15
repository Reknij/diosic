import type { PubMediaInfo } from "~/api/model"
import { Howl, Howler } from 'howler';
import { getMediaFileAddress } from "~/api/media";

export enum PlayMode {
    Order,
    Repeat,
    RepeatOnce,
    Shuffle,
}

export interface MediaPlayerState {
    visible: boolean,
    current: PubMediaInfo | undefined,
    currentIndex: number,
    currentElapsedSeconds: number,
    _elapsedSecondsHandler: number,
    _setSeekHandler: number,
    playlist: PubMediaInfo[],
    playing: boolean,
    mode: PlayMode
}

export const useMediaPlayerState = () => useState<MediaPlayerState>('mediaPlayer', () => {
    const state: MediaPlayerState = {
        visible: false,
        playing: false,
        current: undefined,
        currentIndex: -1,
        currentElapsedSeconds: 0,
        _elapsedSecondsHandler: -1,
        _setSeekHandler: -1,
        playlist: [],
        mode: PlayMode.Order,
    };
    return state;
})

const useSound = () => useState<Howl | undefined>('howlerSound')
function unloadHowl() {
    const sound = useSound().value;
    if (sound) {
        sound.stop();
        sound.off();
        sound.unload();
        useSound().value = undefined;
    }
}

function setElapsedTime(clearOnly: boolean = false) {
    const state = useMediaPlayerState().value;
    if (state._elapsedSecondsHandler != -1) {
        window.clearInterval(state._elapsedSecondsHandler);
        state._elapsedSecondsHandler = -1;
    }
    if (!clearOnly) {
        state._elapsedSecondsHandler = window.setInterval(() => {
            state.currentElapsedSeconds = useSound().value?.seek() ?? 0;
        }, 10);
    }
}

function loadHowl() {
    const state = useMediaPlayerState().value;
    if (state.current) {
        unloadHowl();
        const sound = new Howl({
            src: getMediaFileAddress(state.current.id),
            format: state.current.file_type,
            onplay() {
                state.playing = true;
                setElapsedTime();
            },
            onstop() {
                state.playing = false;
                setElapsedTime(true);
            },
            onpause() {
                state.playing = false;
            },
            onend(soundId) {
                state.playing = false;
                const player = useMediaPlayer();
                const playNext = (loop = false) => {
                    if (player.canForward()) {
                        player.forward();
                    } else if (loop) {
                        player.playByIndex(0);
                    } else {
                    }
                }
                switch (state.mode) {
                    case PlayMode.Order:
                        playNext();
                        break;
                    case PlayMode.Repeat:
                        playNext(true);
                        break;
                    case PlayMode.RepeatOnce:
                        sound.play(soundId)
                        break;
                    case PlayMode.Shuffle:
                        if (player.total() > 0) {
                            const random = Math.floor(Math.random() * state.playlist.length);
                            player.playByIndex(random);
                        } else {
                            state.currentIndex = -1;
                        }
                        break;
                    default:
                        break;
                }
            },
            html5: true,
            autoplay: true,
        })
        useSound().value = sound;
    }
}

export const useMediaPlayer = () => {
    const state = useMediaPlayerState();
    return {
        getCurrent(): PubMediaInfo | undefined {
            if (state.value.currentIndex > -1 && state.value.currentIndex < this.total()) {
                return state.value.playlist[state.value.currentIndex];
            }
            return undefined;
        },
        show() {
            state.value.visible = true;
        },
        hide() {
            state.value.visible = false;
        },
        play(media: PubMediaInfo, show: boolean = false) {
            this.stop();
            state.value.playlist = [media];
            this.playByIndex(0);
            if (show) this.show();
        },
        playByIndex(index: number) {
            if (this.total() > 0 && this.total() > index && index >= 0) {
                state.value.currentIndex = index;
                state.value.current = state.value.playlist[index];
                loadHowl();
            }
        },
        playList(list: PubMediaInfo[], mode: PlayMode = PlayMode.Order) {
            this.stop();
            this.setMode(mode);
            state.value.playlist = list;
            if (mode === PlayMode.Shuffle) {
                const random = Math.floor(Math.random() * state.value.playlist.length);
                this.playByIndex(random);
            } else {
                this.playByIndex(0);
            }
        },
        setMode(mode: PlayMode) {
            state.value.mode = mode;
        },
        total() {
            return state.value.playlist.length;
        },
        stop() {
            state.value.currentIndex = -1;
            state.value.current = undefined;
            state.value.mode = PlayMode.Order;
            unloadHowl();
        },
        resume() {
            useSound().value?.play();
        },
        pause() {
            useSound().value?.pause();
        },
        canForward() {
            return this.total() > 0 && state.value.currentIndex < (this.total() - 1);
        },
        forward() {
            if (this.canForward()) {
                this.playByIndex(state.value.currentIndex + 1)
            } else {
                console.warn("Current media is last one already or play list is empty.")
            }
        },
        canBackward() {
            return this.total() > 0 && state.value.currentIndex > 0
        },
        backward() {
            if (this.canBackward()) {
                this.playByIndex(state.value.currentIndex - 1)
            } else {
                console.warn("Current media is first one already or play list is empty.")
            }
        },
        skipTo(media: PubMediaInfo) {
            const i = state.value.playlist.indexOf(media);
            if (i != -1) {
                this.playByIndex(i)
            }
        },
        seekTo(seconds: number) {
            const sound = useSound().value;
            const state = useMediaPlayerState().value;
            if (sound) {
                this.pause();
                sound.seek(seconds);
                if (state._setSeekHandler != -1) {
                    window.clearTimeout(state._setSeekHandler);
                }
                state._setSeekHandler = window.setTimeout(() => {
                    this.resume();
                }, 100);
            }
        },
        push(media: PubMediaInfo, target = -1) {
            const i = state.value.playlist.indexOf(media);
            if (i != -1) {
                state.value.playlist.splice(i, 1);
                if (i < target) target -= 1;
            }
            if (target != -1 && this.total() > target) {
                state.value.playlist.splice(target, 0, media);
                return;
            }
            state.value.playlist.push(media);
        },
        clear() {
            if (this.total() > 0) {
                state.value.playlist.splice(0, this.total());
            }
        },
        remove(media: PubMediaInfo) {
            const i = state.value.playlist.indexOf(media);
            if (i != -1) {
                let toIndex = -1;
                if (i === state.value.currentIndex) {
                    if (this.canForward()) {
                        toIndex = state.value.currentIndex;
                    } else if (this.canBackward()) {
                        toIndex = state.value.currentIndex - 1;
                    }
                    this.stop();
                }
                state.value.playlist.splice(i, 1);
                if (toIndex != -1) {
                    this.playByIndex(toIndex);
                }
            }
        }
    }
}