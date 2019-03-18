//
//  XiEventHandler.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Foundation

class EventHandler {
    let _inner: OpaquePointer

    init(callback: @escaping (@convention(c) (UInt32) -> Void)) {
        _inner = xiEventHandlerCreate(callback)
    }

    func handleInput(val: UInt32) -> Bool {
        return xiEventHandlerHandleInput(_inner, val) != 0
    }

    deinit {
        xiEventHandlerFree(_inner)
    }
}
