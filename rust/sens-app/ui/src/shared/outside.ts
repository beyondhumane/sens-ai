import { commands } from "../ipc/commands";
import { warn } from "../features/chat/state";

export const openOutside = (target: string) => commands.openExternal(target).catch((reason) => warn(String(reason)));
