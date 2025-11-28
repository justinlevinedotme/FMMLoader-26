<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button";
    import { onDestroy, onMount } from "svelte";
    import I18n from "@razein97/tauri-plugin-i18n";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";

    let n18n = $state<I18n | undefined>(undefined);

    let locales = $state([]) as string[];
    let locale = $state("");

    let unlisten: UnlistenFn;

    onMount(async () => {
        const e18n = I18n.getInstance();
        await e18n.load();

        n18n = e18n;

        locales = await I18n.getAvailableLocales();
        locale = locales[0];

        unlisten = await listen<string>("i18n:locale_changed", (event) => {
            locale = event.payload;
        });
    });

    onDestroy(() => {
        n18n?.destroy();
        unlisten();
    });
</script>

<main class="w-full h-full flex flex-col items-center space-y-4 p-4">
    <h1 class="font-bold text-center">Tauri Plugin i18n</h1>
    <div
        class="w-full border rounded-md shadow-xs flex flex-row p-2 space-x-4 items-center justify-center"
    >
        <p data-i18n="root.select_language"></p>

        <select
            bind:value={locale}
            name="locales"
            id="locales"
            onchange={async () => {
                await I18n.setLocale(locale);
                // console.log(locale);
            }}
        >
            {#each locales as locale}
                <option value={locale}>{locale}</option>
            {/each}
        </select>

        <span class="w-32"></span>

        <Button
            data-i18n="root.select_language"
            onclick={async () => {
                await invoke("open_custom_menu");
            }}>Native menu</Button
        >
    </div>

    <div
        class="w-full border rounded-md shadow-xs flex flex-row p-2 space-x-4 items-center justify-center"
    >
        <p data-i18n="root.para"></p>
    </div>
</main>

<style>
</style>
