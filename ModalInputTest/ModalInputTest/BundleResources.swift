//
//  BundleResources.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-15.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

let BUILD_SCRIPT_FILENAME = "cargo_build.sh"

/// Simpler management of bundled resources
class BundleResources {
    private static let shared = BundleResources()

    static var buildScriptURL: URL {
        return shared.buildScriptURL
    }

    lazy var buildScriptURL: URL = {
        return Bundle.main.url(forResource: "cargo_build", withExtension: "sh")!
    }()
}
