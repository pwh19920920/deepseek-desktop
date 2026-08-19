/**
 * The configurable-plugins tab's card list.
 *
 * The tab dispatches its slot by settings namespace, so what it renders is
 * the intersection of two ledgers: the namespaces the Host serves and the
 * cards registered into `settings.plugin.item`. A served namespace no card
 * claims renders nothing — another surface owns it, or this deployment ships
 * no browser half for it — and a card whose namespace the Host does not serve
 * is never dispatched, so a plugin this deployment did not compose leaves no
 * trace and does not count toward the empty line.
 */
import type { IApiClient } from '@deepseek-ai/dsh-client-connection/client';
import type { StoredEntry } from '@deepseek-ai/dsh-client-ui-slots';
import { type SnapshotStore } from '@deepseek-ai/dsh-client-runtime/client';
/** What the section renders. */
export interface ConfigurablePluginsTabState {
    /**
     * Whether the Host has answered once. The empty line waits for it: an
     * unanswered read is not the same statement as "this deployment configures
     * no plugin", and saying the second while the first is true would flash a
     * wrong answer on every open.
     */
    loaded: boolean;
    /**
     * Namespaces to dispatch, in the order their cards registered, narrowed to
     * those the Host serves. Card registration order rather than the Host's
     * description order: the latter follows plugin activation, which async
     * settings injection can reorder between boots, and a settings page whose
     * cards move between visits is worse than one whose order a registrant
     * chose.
     */
    namespaces: string[];
}
/** The registration-side face the tab's slot entry injects. */
export interface ConfigurablePluginsTabFace {
    hooks: {
        /** Section snapshot bound by the renderer as usePluginConfigSection. */
        configurablePlugins: SnapshotStore<ConfigurablePluginsTabState>;
    };
}
/** Reads the served namespaces and pairs them with the cards that claim them. */
export declare class ConfigurablePluginsTabController {
    private readonly api;
    private readonly entries;
    private readonly store;
    /** Last Host answer; kept so a slot mutation republishes without a wire read. */
    private served;
    private loaded;
    private generation;
    private disposed;
    /**
     * @param api - settings wire face.
     * @param entries - reads the cards currently registered into the section's slot.
     */
    constructor(api: Pick<IApiClient, 'settings'>, entries: () => readonly StoredEntry[]);
    /** Opaque read of {@link disposed}: control flow cannot narrow it across awaits. */
    private isDisposed;
    /**
     * Re-read the served namespaces from the Host and republish.
     * @returns settlement after the read, or immediately once disposed.
     */
    load(): Promise<void>;
    /** Republish after the slot ledger changed; a card registered late joins here. */
    refresh(): void;
    /** Stop publishing; an in-flight read settles without touching the store. */
    dispose(): void;
    /**
     * Build the face the tab's slot registration injects.
     * @returns the tab's snapshot source.
     */
    inject(): ConfigurablePluginsTabFace;
    private publish;
}
//# sourceMappingURL=tab-store.d.ts.map