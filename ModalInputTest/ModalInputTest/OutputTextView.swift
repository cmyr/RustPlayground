//
//  OutputTextView.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-19.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class OutputTextView: NSTextView {

    override var isFlipped: Bool {
        return true
    }

    var lockScrollToBottom: Bool = true

    override func layout() {
        super.layout()
        if lockScrollToBottom {
            let bottom = CGRect(x: 0, y: bounds.maxY - 2, width: 2, height: 2)
            scrollToVisible(bottom)
        }
    }
}
