import { Plugin } from "obsidian";

/** The mdkernel plugin (P1-E). E0: an empty plugin that loads. */
export default class MdkernelPlugin extends Plugin {
  override onload(): void {
    // panels and commands arrive in E1–E3
  }
}
