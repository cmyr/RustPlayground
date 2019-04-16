//
//  Runner.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-15.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Foundation

/// Responsible for compiling and running a snippet.
class Runner {
    //TODO: not this
    let workDirectory = URL(string: "/Users/rofls/dev/hacking/macos_rustplay_test")!
    let scriptPath: URL
    let fileName: String
    let targetName: String

    init(scriptPath: URL, fileName: String) {
        self.scriptPath = scriptPath
        self.fileName = fileName
        self.targetName = URL(string: fileName)!.deletingPathExtension().path
    }

    func compile() -> Bool {
        let task = Process()
        task.launchPath = scriptPath.path
        task.currentDirectoryPath = workDirectory.path
        //        task.arguments = ["rustc", fileName]
        let errPipe = Pipe()
        let outPipe = Pipe()

        task.standardError = errPipe
        task.standardOutput = outPipe

        outPipe.fileHandleForReading.readabilityHandler = { handle in
            let data = handle.availableData
            if let errString = String(data: data, encoding: .utf8) {
                print(errString, terminator: "")
            }
        }

        errPipe.fileHandleForReading.readabilityHandler = { handle in
            let data = handle.availableData
            if let errString = String(data: data, encoding: .utf8) {
                print(errString, terminator: "")
            }
        }

        task.launch()
        task.waitUntilExit()
        if task.terminationStatus != 0 {
            print("build failed")
        }
        return task.terminationStatus == 0
    }

    func run() {
        let targetPath = workDirectory.appendingPathComponent(targetName, isDirectory: false).path
        let task = Process()
        task.launchPath = targetPath
        task.currentDirectoryPath = workDirectory.path
        let errPipe = Pipe()
        let outPipe = Pipe()

        task.standardError = errPipe
        task.standardOutput = outPipe

        outPipe.fileHandleForReading.readabilityHandler = { handle in
            let data = handle.availableData
            if let errString = String(data: data, encoding: .utf8) {
                print(errString, terminator: "")
            }
        }

        errPipe.fileHandleForReading.readabilityHandler = { handle in
            let data = handle.availableData
            if let errString = String(data: data, encoding: .utf8) {
                print(errString, terminator: "")
            }
        }
        task.launch()
        task.waitUntilExit()
    }
}