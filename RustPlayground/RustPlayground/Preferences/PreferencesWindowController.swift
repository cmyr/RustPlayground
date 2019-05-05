//
//  PreferencesWindowController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-19.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class PreferencesWindowController: NSWindowController {

    override func windowDidLoad() {
        super.windowDidLoad()

        //HACK: we have to set this to resizeable for the tabview to size correctly
        // (or at least I couldn't figure out another way, and don't want to spend two more hours on it)
        var mask = window!.styleMask
        mask.remove(.resizable)
        window?.styleMask = mask
    }
}
