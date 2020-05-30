<script>
    import wasm from "../../wasm/Cargo.toml";

    let camouflage;
    let decamouflage;

    async function importWasm() {
        const exports = await wasm();
        camouflage = exports.camouflage;
        decamouflage = exports.decamouflage;
    }

    let firstTab = true;

    let payload = "";
    let dummy = "";
    let key = "";
    let compressionLevel = 10;
    let camouflaged = "";

    let notification = "";
    let success = true;

    let loading = false;

    function hide() {
        loading = true;
        try {
            camouflaged = camouflage(
                payload,
                dummy,
                key.length > 0 ? key : undefined,
                compressionLevel
            );
            success = true;
            notification = "Payload successfully hidden";
            firstTab = false;
        } catch (err) {
            success = false;
            notification = err.toString();
        }
        loading = false;
    }

    function retrieve() {
        loading = true;
        try {
            payload = decamouflage(
                camouflaged,
                key.length > 0 ? key : undefined
            );
            success = true;
            notification = "Payload successfully retrieved";
            firstTab = true;
        } catch (err) {
            success = false;
            notification = err.toString();
        }
        loading = false;
    }
</script>

<main>
    {#await importWasm()}
        <div class="loading loading-lg"></div>
    {:then _}
        {#if notification.length > 0}
            <div class="toast {success ? 'toast-success' : 'toast-error'}">
                <button
                    class="btn btn-clear float-right"
                    on:click="{() => (notification = '')}"
                ></button>
                {notification}
            </div>
        {/if}
        <div class="card">
            <div class="card-header">
                <ul class="tab tab-block">
                    <li class="tab-item {firstTab ? 'active' : ''}">
                        <a href="#" on:click="{() => (firstTab = true)}">
                            Hide
                        </a>
                    </li>
                    <li class="tab-item {firstTab ? '' : 'active'}">
                        <a href="#" on:click="{() => (firstTab = false)}">
                            Retrieve
                        </a>
                    </li>
                </ul>
            </div>
            {#if firstTab}
                <div class="card-body">
                    <div class="form-group">
                        <label class="form-label" for="payload-input">
                            Payload
                        </label>
                        <textarea
                            class="form-input"
                            id="payload-input"
                            rows="4"
                            bind:value="{payload}"
                        ></textarea>
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="dummy-input">
                            Dummy
                        </label>
                        <textarea
                            class="form-input"
                            id="dummy-input"
                            rows="2"
                            bind:value="{dummy}"
                        ></textarea>
                    </div>
                    <div class="columns">
                        <div class="column col-8">
                            <div class="form-group">
                                <label class="form-label" for="keyh-input">
                                    Key
                                </label>
                                <input
                                    type="password"
                                    class="form-input"
                                    id="keyh-input"
                                    bind:value="{key}"
                                />
                            </div>
                        </div>
                        <div class="column col-4">
                            <div class="form-group">
                                <label class="form-label" for="cl-input">
                                    Compression level
                                </label>
                                <input
                                    type="number"
                                    class="form-input"
                                    id="cl-input"
                                    min="0"
                                    max="11"
                                    bind:value="{compressionLevel}"
                                />
                            </div>
                        </div>
                    </div>
                </div>
                <div class="card-footer">
                    <button
                        id="hide"
                        class="btn btn-primary {loading ? 'loading' : ''}"
                        on:click="{hide}"
                        disabled="{loading}"
                    >
                        Hide
                    </button>
                </div>
            {:else}
                <div class="card-body">
                    <div class="form-group">
                        <label class="form-label" for="hidden-input">
                            Hidden
                        </label>
                        <textarea
                            class="form-input"
                            id="hidden-input"
                            rows="2"
                            bind:value="{camouflaged}"
                        ></textarea>
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="keyr-input">Key</label>
                        <input
                            type="password"
                            class="form-input"
                            id="keyr-input"
                            bind:value="{key}"
                        />
                    </div>
                </div>
                <div class="card-footer">
                    <button
                        id="retrieve"
                        class="btn btn-primary {loading ? 'loading' : ''}"
                        on:click="{retrieve}"
                        disabled="{loading}"
                    >
                        Retrieve
                    </button>
                </div>
            {/if}
        </div>
    {:catch err}
        <div class="empty">
            <p class="empty-title h5">
                An error occurred while loading the WebAssembly module
            </p>
            <p class="empty-subtitle">{err}</p>
        </div>
    {/await}
</main>

<style>
    main {
        min-width: 480px;
    }

    #hide,
    #retrieve {
        width: 100%;
    }
</style>
