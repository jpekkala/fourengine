const CopyWebpackPlugin = require('copy-webpack-plugin');
const path = require('path');

module.exports = {
    entry: './bootstrap.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'bootstrap.js',
    },
    mode: 'development',
    plugins: [
        new CopyWebpackPlugin({
            patterns: [{
                from: 'public/index.html',
            }, {
                from: '../../books/7x6-ply4.txt',
            }, {
                from: '../../books/7x6-ply8.txt',
            }]
        })
    ],
    experiments: {
        asyncWebAssembly: true,
    }
};
