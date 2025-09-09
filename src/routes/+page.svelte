<script lang="ts">
    import { appConfigDir, join } from "@tauri-apps/api/path";
    import {
        exists,
        mkdir,
        writeTextFile,
        readTextFile,
    } from "@tauri-apps/plugin-fs";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { onMount } from "svelte";
    import DiGithubBadge from "svelte-icons/di/DiGithubBadge.svelte";
    import FaDownload from "svelte-icons/fa/FaDownload.svelte";
    import MdDeleteForever from "svelte-icons/md/MdDeleteForever.svelte";
    import FaFolder from "svelte-icons/fa/FaFolder.svelte";
    import FaPlayCircle from "svelte-icons/fa/FaPlayCircle.svelte";
    import FaRegPlayCircle from "svelte-icons/fa/FaRegPlayCircle.svelte";

    let modData: {
        name: string;
        author: string;
        version: string;
        file: string;
        extractTo: string;
        toDelete: string;
    }[] = $state([]);

    const defaultConfig = {
        ss_path: "",
    };

    let configDir: string;
    let configFile: string;

    let config = $state(defaultConfig);
    let ssbp = $derived(
        config.ss_path.replaceAll("Hollow Knight Silksong.exe", ""),
    );
    let ssPlugins = $derived(`${ssbp}BepInEx\\plugins`);

    let doesBepInExExist = $derived(doesBepinAlrExist(config));

    async function configExists() {
        const dirExists = await exists(configDir);

        const fileExists = await exists(configFile);

        return dirExists && fileExists;
    }

    async function createDefaults() {
        if (!(await configExists())) {
            await mkdir(configDir, { recursive: true });
            await writeTextFile(configFile, JSON.stringify(defaultConfig), {
                create: true,
            });
        }
    }

    async function selectSilksong() {
        await createDefaults();

        const exePath = await open({
            multiple: false,
            filters: [
                {
                    name: "Hollow Knight Silksong.exe",
                    extensions: ["exe"],
                },
            ],
            defaultPath:
                "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Hollow Knight Silksong",
        });

        if (!exePath) {
            throw new Error("No executable selected");
        }

        config.ss_path = exePath;

        await writeTextFile(configFile, JSON.stringify(config));
    }

    async function invokeDownloadBepinEx() {
        await invoke("download_and_extract_bepinex", {
            ssPath: ssbp,
        });
    }

    async function invokeDeleteBepinEx() {
        await invoke("delete_bepinex_files", { ssPath: ssbp });
    }

    async function doesBepinAlrExist(config: any) {
        let bepinFolder = await join(ssbp, "BepInEx");
        let doorstop = await join(ssbp, "winhttp.dll");
        let disableddoorstop = await join(ssbp, "winhttp.disabled");

        let bepinFolderExists = await exists(bepinFolder);
        let doorstopExists =
            (await exists(doorstop)) || (await exists(disableddoorstop));

        let isBepinInstalled = bepinFolderExists && doorstopExists;
        return isBepinInstalled;
    }

    async function downloadMod(url: string, fileName: string) {
        let ssPluginPath = `${ssPlugins}\\${fileName}`;
        await invoke("download_command", { url, savePath: ssPluginPath });
    }

    async function extract_mod(path: string, extractTo: string) {
        await invoke("extract_zip", { zipPath: path, destPath: extractTo });
    }

    async function deleteMod(deletePath: string) {
        console.log(deletePath);
        await invoke("delete_mod", { path: deletePath });
    }

    async function modFileExists(mod: any): Promise<boolean> {
        if (mod.toDelete) {
            return await exists(`${ssPlugins}\\${mod.toDelete}`);
        } else {
            return await exists(`${ssPlugins}\\${mod.file}`);
        }
    }

    async function launchModdedGame() {
        invoke("open_game", { path: config.ss_path, args: [] });
    }

    async function launchVanillaGame() {
        invoke("open_game", {
            path: config.ss_path,
            args: ["--doorstop-enabled", "false"],
        });
    }

    onMount(async () => {
        configDir = await appConfigDir();
        configFile = await join(configDir, "config.json");

        await createDefaults();

        config = JSON.parse(await readTextFile(configFile));

        modData = await fetch(
            "https://raw.githubusercontent.com/GamingInfinite/Needle/refs/heads/main/src/lib/modlinks.json",
        ).then((res) => res.json());
    });
</script>

<div class="flex flex-row m-4">
    <div class="flex flex-col w-3/8">
        <div class="flex flex-row">
            <button
                class="btn btn-ghost w-full justify-start"
                onclick={launchModdedGame}
            >
                <div class="w-7 h-7"><FaPlayCircle></FaPlayCircle></div>
                Launch Modded
            </button>
        </div>
        <div class="flex flex-row">
            <button
                class="btn btn-ghost w-full justify-start"
                onclick={launchVanillaGame}
            >
                <div class="w-7 h-7"><FaRegPlayCircle></FaRegPlayCircle></div>
                Launch Vanilla
            </button>
        </div>
        <div class="flex flex-row">
            {#await doesBepInExExist then isBepinReal}
                {#if isBepinReal}
                    <button
                        class="btn btn-ghost w-full justify-start"
                        disabled={config.ss_path == "" && isBepinReal}
                        onclick={async () => {
                            await invokeDeleteBepinEx();
                        }}
                    >
                        <div class="w-8 h-8">
                            <MdDeleteForever></MdDeleteForever>
                        </div>
                        Un-Install BepinEx
                    </button>
                {:else}
                    <button
                        class="btn btn-ghost w-full justify-start"
                        disabled={config.ss_path == "" && isBepinReal}
                        onclick={async () => {
                            await invokeDownloadBepinEx();
                        }}
                    >
                        <div class="w-8 h-8"><FaDownload></FaDownload></div>
                        Install BepinEx
                    </button>
                {/if}
            {/await}
        </div>
        <div class="flex flex-row">
            <button
                class="btn btn-ghost w-full justify-start"
                onclick={selectSilksong}
            >
                <div class="w-8 h-8"><FaFolder></FaFolder></div>
                Select Silksong Install
            </button>
        </div>
    </div>
    <div class="divider divider-horizontal"></div>
    <div class="flex flex-col gap-2 w-full">
        {#each modData as mod}
            <div class="flex flex-row gap-10 items-center justify-between">
                <div class="flex flex-row gap-2 items-center">
                    <div class="font-bold text-lg">
                        {mod.name}
                    </div>
                    <div>
                        {#if !mod.version.toLowerCase().includes("v")}
                            v
                        {/if}{mod.version}
                    </div>
                    <div>
                        by {mod.author}
                    </div>
                    <a
                        class="btn btn-circle w-10 h-10"
                        href={`https://github.com/${mod.author}/${mod.name}`}
                        ><DiGithubBadge></DiGithubBadge></a
                    >
                </div>
                <div>
                    {#await modFileExists(mod) then exists}
                        <input
                            type="checkbox"
                            checked={exists}
                            class="toggle"
                            onchange={async (e) => {
                                if (e.currentTarget.checked) {
                                    // Downloading
                                    let download_link = `https://github.com/${mod.author}/${mod.name}/releases/download/${mod.version}/${mod.file}`;
                                    await downloadMod(download_link, mod.file);
                                    if (mod.file.includes(".zip")) {
                                        await extract_mod(
                                            `${ssPlugins}\\${mod.file}`,
                                            `${ssbp}\\${mod.extractTo}`,
                                        );
                                    }
                                } else {
                                    // Delete Mod... wish there was a better way of disabling stuff.
                                    let deletePath = `${ssPlugins}\\`;
                                    if (mod.toDelete) {
                                        deletePath += mod.toDelete;
                                    } else {
                                        deletePath += mod.file;
                                    }
                                    await deleteMod(deletePath);
                                }
                            }}
                        />
                    {/await}
                </div>
            </div>
        {/each}
    </div>
</div>
