const zip = require('deterministic-zip');

zip('luct', 'luct.xpi', { cwd: 'luct' }, (err) => {
    if (err) {
        console.log(err);
    } else {
        console.log('Done!');
    }
});