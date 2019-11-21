//
//  Runner.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-15.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Foundation

protocol RunnerOutputHandler {
    func printInfo(text: String);
    func handleStdOut(text: String);
    func handleStdErr(text: String);
}

func runCommand(atPath path: String, handler: RunnerOutputHandler) {
    handler.printInfo(text: "Running")

    let task = Process()
    task.launchPath = path

    let errPipe = Pipe()
    let outPipe = Pipe()

    task.standardError = errPipe
    task.standardOutput = outPipe

    outPipe.fileHandleForReading.readabilityHandler = { handle in
        let data = handle.availableData
        if let outString = String(data: data, encoding: .utf8) {
            if outString.count > 0 {
                handler.handleStdOut(text: outString)
            }
        }
    }

    errPipe.fileHandleForReading.readabilityHandler = { handle in
        let data = handle.availableData
        if let errString = String(data: data, encoding: .utf8) {
            if errString.count > 0 {
                handler.handleStdErr(text: errString)
            }
        }
    }

    task.launch()
    task.waitUntilExit()
}
