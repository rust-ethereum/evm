pipeline {
    agent none
    stages {
        stage('Install Rust') {
            parallel {
                stage('macos') {
                    agent { label 'macos' }
                    steps {
                        sh './ci/install_rust.sh'
                    }
                }
                stage('linux') {
                    agent { label 'linux' }
                    steps {
                        sh './ci/install_rust.sh'
                    }
                }
                stage('windows') {
                    agent { label 'windows' }
                    steps {
                        powershell '& ./ci/install_rust.ps1'
                    }
                }
            }
        }
        stage('Build') {
            parallel {
                stage('macos') {
                    agent {
                        label 'macos'
                    }
                    stages {
                        stage('stable') {
                            steps {
                                sh './ci/build.sh'
                            }
                        }
                        stage('beta') {
                            steps {
                                sh './ci/build.sh +beta'
                            }
                        }
                        stage('nightly') {
                            steps {
                                sh './ci/build.sh +nightly'
                            }
                        }
                    }
                }
                stage('linux') {
                    agent {
                        label 'linux'
                    }
                    stages {
                        stage('stable') {
                            steps {
                                sh './ci/build.sh'
                            }
                        }
                        stage('beta') {
                            steps {
                                sh './ci/build.sh +beta'
                            }
                        }
                        stage('nightly') {
                            steps {
                                sh './ci/build.sh +nightly'
                            }
                        }
                    }
                }
                stage('windows') {
                    agent {
                        label 'windows'
                    }
                    stages {
                        stage('stable') {
                            steps {
                                bat 'call ./ci/build.bat'
                            }
                        }
                    }
                }
            }
        }
        stage('Test') {
            parallel {
                stage('macos') {
                    agent {
                        label 'macos'
                    }
                    steps {
                        sh './ci/test.sh'
                    }
                }
                stage('linux') {
                    agent {
                        label 'linux'
                    }
                    steps {
                        sh './ci/test.sh'
                    }
                }
                stage('windows') {
                    agent {
                        label 'windows'
                    }
                    steps {
                        bat 'call ./ci/test.bat'
                    }
                }
            }
        }
        stage('Lint') {
            agent { node { label 'macos' } }
            steps {
                sh 'cargo check 2>&1 | tee rustc.build_log'
                sh 'cargo clean'
                sh 'cargo clippy 2>&1 | tee clippy.build_log'
                sh 'if grep -q "^error" clippy.build_log; then echo "clippy found a severe error"; exit 1; fi'
            }
            post {
                always {
                    script {
                        recordIssues enabledForFailure: true,
                            qualityGates: [[threshold: 10, type: 'TOTAL', unstable: true]],
                            healthy: 5, unhealthy: 20, minimumSeverity: 'HIGH',
                            tools: [
                                groovyScript(parserId: 'clippy-warnings', pattern: "clippy.build_log", reportEncoding:'UTF-8'),
                                groovyScript(parserId: 'rustc-warnings', pattern: "rustc.build_log", reportEncoding:'UTF-8')
                            ]
                    }
                }
            }
        }
        stage('Rustfmt') {
            agent {
                label 'macos'
            }
            steps {
                sh 'cargo fmt -- --check'
            }
        }
    }
}
